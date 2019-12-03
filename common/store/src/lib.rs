
use std::path::PathBuf;
use std::env;

extern crate rocksdb;

pub mod Config;
pub mod MapDB;



#[derive(Copy,Clone,Debug,Default)]
pub struct Config {
    pub path: PathBuf,
}

impl Default for Config {
    fn default() -> self {
        let mut cur = env::current_dir().unwrap();
        cur.push("map_db");
        Config{
            path:   cur,
        }
    }
}