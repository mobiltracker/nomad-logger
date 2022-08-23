use log::{Level, Metadata, Record};

use crate::format_log;

pub struct ConsoleLogger {
    max_log_level: Level,
}

pub struct ConsoleLoggerConfig {
    max_log_level: Level,
}

impl ConsoleLoggerConfig {
    pub fn new() -> Self {
        Self {
            max_log_level: Level::Info,
        }
    }
}

impl Default for ConsoleLoggerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleLogger {
    pub fn init(config: ConsoleLoggerConfig) -> Result<(), log::SetLoggerError> {
        let logger = ConsoleLogger {
            max_log_level: config.max_log_level,
        };

        log::set_boxed_logger(Box::new(logger))
            .map(|()| log::set_max_level(config.max_log_level.to_level_filter()))
    }
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_log_level
    }

    fn log(&self, record: &Record) {
        let log = format_log(record);

        if record.level() >= Level::Error {
            eprint!("{}", log);
        } else {
            print!("{}", log);
        }
    }

    fn flush(&self) {}
}
