
use std::sync::{Arc, RwLock};
use rocksdb::{Error, DB};
use crate::config::Config;


pub struct MapDB{
    inner:     Arc<RwLock<DB>>,
}


impl MapDB {
    pub fn open(cfg: Config) -> Result<Self, Error> {
        let db = DB::open_default(&cfg.path).unwrap();
        Ok(MapDB{
            inner:     Arc::new(RwLock::new(db)),
        })
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(),Error> {
        let db = self.inner.write().unwrap();
        db.put(key, value)
    }

    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let db = self.inner.read().unwrap();
        db.get(key)
    }

    pub fn remove(&mut self, key: &[u8]) -> Result<(),Error> {
        let db = self.inner.write().unwrap();
        db.delete(key)
    }

    pub fn exists(&self, key: &[u8]) -> Result<bool, Error> {
        let db = self.inner.read().unwrap();
        db.get(key)
            .map_err(Into::into)
            .and_then(|val| Ok(val.is_some()))
    }
}


#[test]
fn test_set_value() {
    let cfg = Config::default();
    let mut m = MapDB::open(cfg).unwrap();

    assert!(m.put(b"k1", b"v1111").is_ok());

    let r: Result<Option<Vec<u8>>, Error> = m.get(b"k1");

    assert_eq!(r.unwrap().unwrap(), b"v1111");
    assert!(m.remove(b"k1").is_ok());
    assert!(m.get(b"k1").unwrap().is_none());
    assert!(!m.exists(b"k1").unwrap());
}