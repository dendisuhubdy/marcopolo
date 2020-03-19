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

//! MarcoPolo CLI.
extern crate libc;
extern crate signal_hook;

use std::path::PathBuf;
use clap::{App, Arg, SubCommand};
use signal_hook::iterator::Signals;
use logger::LogConfig;
use service::{Service, NodeConfig};
use std::thread;
use std::fmt;

pub fn run() {
    let matches = App::new("map")
        .version("0.1.0")
        .about("MarcoPolo Protocol - A new P2P e-cash system")
        .arg(Arg::with_name("data_dir")
            .long("datadir")
            .short("d")
            .value_name("PATH")
            .takes_value(true)
            .help("Run as if map was started in <PATH> instead of the current working directory."))
        .arg(Arg::with_name("log")
            .long("log")
            .short("l")
            .value_name("LOG_FILTER")
            .takes_value(true)
            .help("Sets logging filter with <LOG_FILTER>."))
        .arg(Arg::with_name("single")
            .long("single")
            .short("s")
            .help("Run with single node"))
        .subcommand(SubCommand::with_name("clean")
            .about("Remove the whole chain data"))
        .get_matches();

    let mut config = NodeConfig::default();

    if let Some(data_dir) = matches.value_of("data_dir") {
        config.data_dir = PathBuf::from(data_dir);
    }

    if let Some(log_filter) = matches.value_of("log") {
        let log_config = LogConfig {
            filter: log_filter.to_string(),
        };
        config.log = log_filter.to_string();
        logger::init(log_config);
    } else {
        logger::init(LogConfig::default());
    }

    if matches.is_present("single") {
        println!("Run map with single node");
    }

    if let Some(_) = matches.subcommand_matches("clean") {
        println!("Remove the whole chain data");
        return;
    }

    let node = Service::new_service(config);
    let (tx, th_handle) = node.start();
    let signals = Signals::new(&[signal_hook::SIGINT,signal_hook::SIGQUIT]).unwrap();
    thread::spawn(move||{
        for sig in &signals {
            match sig as libc::c_int {
                signal_hook::SIGINT | signal_hook::SIGQUIT => {
                    tx.send(1).unwrap();
                    println!("Received signal {:?}", sig);
                    return;
                },
                _ => {},
            };
        }
    }).join().unwrap();
    th_handle.join();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
