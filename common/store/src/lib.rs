
extern crate rocksdb;

pub mod mapdb;

pub mod config {
    use std::path::PathBuf;
    use std::env;

    #[derive(Clone,Debug)]
    pub struct Config {
        pub path: PathBuf,
    }

    impl Default for Config {
        fn default() -> Self {
            let mut cur = env::current_dir().unwrap();
            cur.push("map_db");
            Config{
                path:   cur,
            }
        }
    }
}