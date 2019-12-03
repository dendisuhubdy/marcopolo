
use std::sync::{Arc,RwLock};
use rocksdb::{Error,Options,DB};
use super::Config;


pub struct MapDB{
    inner:     Arc<RwLock<DB>>,
    // inner: Arc<RwLock<Option<Arc<DB>>>>,
}


impl MapDB {
    pub fn open(cfg: Config) -> Result<Self> {
        let db = DB::open_default(&cfg.path).unwrap();
        Ok(MapDB{
            inner:     Arc::new(RwLock::new(db)),
        })
    }
    
    pub fn put(&mut self,key: Option<Vec<u8>>,value: Option<Vec<u8>>) -> Result<(),Error> {
        let &mut db = self.inner.write().unwrap();
        db.put(key,value)
    }

    pub fn get(&mut self,key: Option<Vec<u8>>) -> Result<Option<Vec<u8>> {
        let &mut db = self.inner.read().unwrap();
        db.get(key)
    }

    pub fn remove(&mut self, key: Option<Vec<u8>>) -> Result<(),Error> {
        let &mut db = self.inner.write().unwrap();
        db.delete(key)
    }
}



#[test]
fn test_get_put_value() {
    let cfg = Config::default();
    let m = MapDB::open(cfg);

    assert!(m.put(b"k1", b"v1111").is_ok());

    let r: Result<Option<Vec<u8>>, Error> = m.get(b"k1");

    assert_eq!(r.unwrap().unwrap(), b"v1111");
    assert!(m.remove(b"k1").is_ok());
    assert!(m.get(b"k1").unwrap().is_none());
}