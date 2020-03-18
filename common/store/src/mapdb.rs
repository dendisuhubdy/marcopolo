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

use std::sync::{Arc, RwLock};
use rocksdb::{Error, DB};
use crate::Config;

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

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let db = self.inner.read().unwrap();
        db.get(key).unwrap()
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

    let r: Option<Vec<u8>> = m.get(b"k1");

    assert_eq!(r.unwrap(), b"v1111");
    assert!(m.remove(b"k1").is_ok());
    assert!(m.get(b"k1").is_none());
    assert!(!m.exists(b"k1").unwrap());
}