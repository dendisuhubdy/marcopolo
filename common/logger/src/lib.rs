// Copyright 2019 MarcoPolo Protocol Authors.
// This file is part of MarcoPolo Protocol.

// MarcoPolo Protocol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// MarcoPolo Protocol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with MarcoPolo Protocol.  If not, see <http://www.gnu.org/licenses/>.

extern crate env_logger;
extern crate log;

use env_logger::filter::{Builder, Filter};
use log::{Log, Record, LevelFilter, Metadata, SetLoggerError};

pub struct LogConfig {
    pub filter: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            filter: None,
        }
    }
}

pub struct Logger {
    filter: Filter,
}

impl Logger {
    pub fn new(config: LogConfig) -> Self {
        let mut builder = Builder::from_env("RUST_LOG");

        if let Ok(ref env_filter) = std::env::var("MAP_LOG") {
            builder.parse(env_filter);
        } else if let Some(ref config_filter) = config.filter {
            builder.parse(config_filter);
        }

        Self {
            filter: builder.build(),
        }
    }

    pub fn filter(&self) -> LevelFilter {
        self.filter.filter()
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if self.filter.matches(record) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}


pub fn init(config: LogConfig) -> Result<(), SetLoggerError> {
    let logger = Logger::new(config);
    log::set_max_level(logger.filter());
    log::set_boxed_logger(Box::new(logger))
}