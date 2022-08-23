use std::{io::Write, path::PathBuf};

use std::fs::File;

use std::io::BufWriter;

use std;

use crate::format_log;

use log::{Level, Metadata, Record};

pub enum FileLoggerMessage {
    String(String),
    Flush,
}

pub struct FileLogger {
    pub max_log_level: Level,
    pub log_tx: crossbeam::channel::Sender<FileLoggerMessage>,
}

pub struct FileLoggerBackgroundService {
    pub log_rx: crossbeam::channel::Receiver<FileLoggerMessage>,
    pub current_file: std::io::BufWriter<std::fs::File>,
    pub log_message_count: usize,
    pub max_count: usize,
    pub file_path: PathBuf,
}

impl FileLoggerBackgroundService {
    pub fn run(mut self) {
        loop {
            let log = self.log_rx.recv().expect("Log channel recv error");

            match log {
                FileLoggerMessage::String(log) => {
                    if self.log_message_count > self.max_count {
                        self.current_file.flush().expect("Failed to flush log file");

                        let mut current_file = self
                            .current_file
                            .into_inner()
                            .expect("failed to get inner log file");

                        current_file.flush().expect("Failed to flush inner file");
                        std::mem::drop(current_file);
                        std::fs::copy(
                            &self.file_path,
                            format!("{}.old", &self.file_path.to_string_lossy()),
                        )
                        .ok();

                        let new_file =
                            File::create(&self.file_path).expect("Failed to roll log file");

                        self.current_file = BufWriter::new(new_file);
                        self.log_message_count = 0;
                    }

                    self.current_file
                        .write_all(log.as_bytes())
                        .expect("Log file failed to write");
                    self.log_message_count += 1;
                }
                FileLoggerMessage::Flush => {
                    self.current_file.flush().expect("log file failed to flush");
                }
            }
        }
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log = format_log(record);
            match self.log_tx.try_send(FileLoggerMessage::String(log)) {
                Ok(()) => {}
                Err(err) => eprintln!("{}", err),
            }
        }
    }

    fn flush(&self) {
        match self.log_tx.try_send(FileLoggerMessage::Flush) {
            Ok(()) => {}
            Err(err) => eprintln!("{}", err),
        }
    }
}

pub struct FileLoggerConfig {
    pub max_log_level: Level,
    pub path: PathBuf,
    pub max_line_count: usize,
}

impl FileLoggerConfig {
    pub fn new(path: PathBuf) -> Self {
        Self {
            max_log_level: Level::Info,
            path,
            max_line_count: 100000,
        }
    }
}

impl FileLogger {
    pub fn init(config: FileLoggerConfig) -> Result<(), log::SetLoggerError> {
        let (tx, rx) = crossbeam::channel::bounded(4096);
        let log_watcher = FileLogger {
            log_tx: tx,
            max_log_level: config.max_log_level,
        };

        let log_file = std::io::BufWriter::new(
            File::create(config.path.clone()).expect("Failed to create log file"),
        );

        let backgroud_writer = FileLoggerBackgroundService {
            log_rx: rx,
            current_file: log_file,
            log_message_count: 0,
            max_count: config.max_line_count,
            file_path: config.path,
        };

        std::thread::spawn(move || backgroud_writer.run());

        log::set_boxed_logger(Box::new(log_watcher))
            .map(|()| log::set_max_level(config.max_log_level.to_level_filter()))
    }
}
