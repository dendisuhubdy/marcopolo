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

extern crate rocksdb;
pub mod mapdb;
pub type Error = rocksdb::Error;
pub type WriteBatch = rocksdb::WriteBatch;

use std::path::PathBuf;
use std::env;
use std::io;

pub trait KVDB {
    fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>>;

    fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()>;

    fn remove(&mut self, key: &[u8]) -> io::Result<()>;
}

#[derive(Clone,Debug)]
pub struct Config {
    pub path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let mut cur = env::current_dir().unwrap();
        cur.push("mapdata");
        Config{
            path:   cur,
        }
    }
}

impl Config {
    pub fn new(mut dir: PathBuf) -> Self {
        dir.push("mapdata");
        Config {
            path: dir,
        }
    }
}
