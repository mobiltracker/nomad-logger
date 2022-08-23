pub mod console_logger;
pub mod file_logger;

use std::path::{Path, PathBuf};

use console_logger::{ConsoleLogger, ConsoleLoggerConfig};
use file_logger::{FileLogger, FileLoggerConfig};
use log::{Level, Record};

pub enum ServiceLoggerKind {
    ConsoleLogger,
    FileLogger,
}

pub struct ServiceLogger {}

pub struct ServiceLoggerEnv {
    log_path: Option<String>,
    max_line_count: Option<usize>,
    log_kind: ServiceLoggerKind,
}

impl ServiceLoggerEnv {
    pub fn from_env() -> Self {
        let log_kind = match std::env::var("LOG_KIND") {
            Ok(log_kind) if log_kind == "FILE" => ServiceLoggerKind::FileLogger,
            _ => ServiceLoggerKind::ConsoleLogger,
        };

        Self {
            log_path: std::env::var("LOG_PATH").ok(),
            max_line_count: std::env::var("MAX_LINE_COUNT")
                .map(|l| l.parse::<usize>().expect("FAILED TO PARSE MAX_LINE_COUNT"))
                .ok(),
            log_kind,
        }
    }
}

impl ServiceLogger {
    pub fn init_from_env() -> Result<(), log::SetLoggerError> {
        let env = ServiceLoggerEnv::from_env();
        match env.log_kind {
            ServiceLoggerKind::ConsoleLogger => Self::init_console(),
            ServiceLoggerKind::FileLogger => {
                let path = env.log_path.expect("MISSING LOG_PATH FOR FILE LOGGER");
                let path = PathBuf::from(path);
                if let Some(folder) = path.parent() {
                    if !folder.exists() {
                        println!("creating {} folder", folder.to_string_lossy());
                        std::fs::create_dir_all(folder).unwrap_or_else(|_| {
                            panic!("Failed to create {} folder", folder.to_string_lossy())
                        });
                    }
                }

                Self::init_file(path, env.max_line_count)
            }
        }
    }

    pub fn init_console() -> Result<(), log::SetLoggerError> {
        ConsoleLogger::init(ConsoleLoggerConfig::new())
    }

    pub fn init_file(
        path: impl AsRef<Path>,
        max_line_count: Option<usize>,
    ) -> Result<(), log::SetLoggerError> {
        FileLogger::init(FileLoggerConfig {
            max_log_level: Level::Info,
            path: path.as_ref().to_owned(),
            max_line_count: max_line_count.unwrap_or(100_000),
        })
    }
}

#[inline(always)]
pub fn format_log(record: &Record) -> String {
    format!("{}\n", record.args())
}
