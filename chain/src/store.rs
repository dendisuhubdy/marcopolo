use map_store::mapdb::MapDB;
use map_store::config::Config;
use map_store::Error;

const HEADER_PREFIX: u8 = 'h' as u8;
const HEAD_PREFIX: u8 = 'H' as u8;
const BLOCK_PREFIX: u8 = 'b' as u8;

/// Blockchain storage backend implement
pub struct ChainDB {
    db: MapDB,
}

impl ChainDB {
    pub fn new() -> Result<Self, Error> {
        let cfg = Config::default();
        let mut m = MapDB::open(cfg).unwrap();

        Ok(ChainDB{db: m})
    }
}
