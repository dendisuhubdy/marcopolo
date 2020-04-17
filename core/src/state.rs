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
use serde::{Serialize, Deserialize};
use bincode;
use hash_db::{HashDB, AsHashDB, Prefix};
use trie_db::DBValue;
use map_store::mapdb::MapDB;
use map_store::KVDB;
use crate::types::Hash;
use crate::trie::{MemoryDB, EMPTY_TRIE, Blake2Hasher};

#[derive(Clone)]
pub struct CachingDB {
    backend: Arc<RwLock<dyn KVDB>>,
    cached: MemoryDB,
}

impl CachingDB {
    /// Create a storage backend of trie structure along with memory caching
    pub fn new(backend: Arc<RwLock<dyn KVDB>>) -> Self {
        CachingDB {
            backend: backend,
            cached: MemoryDB::new(EMPTY_TRIE),
        }
    }

    fn payload(&self, key: &Hash) -> Option<Payload> {
        if let Some(data) = self.backend.read().unwrap().get(key.as_bytes()).unwrap() {
            let value: Payload = bincode::deserialize(&data.as_slice()).unwrap();
            Some(value)
        } else {
            None
        }
    }

    /// Write memory changes to backend db
    pub fn commit(&mut self) {
        for i in self.cached.drain() {
            let (key, (value, rc)) = i;
            if rc != 0 {
                match self.payload(&key) {
                    Some(x) => {
                        let total_rc: i32 = x.count as i32 + rc;
                        if total_rc < 0 {
                            panic!("negtive count of trie item");
                        }
                        let encoded = bincode::serialize(&Payload::new(total_rc as u32, x.value)).unwrap();
                        let mut backend = self.backend.write().unwrap();
                        backend.put(key.as_bytes(), &encoded).expect("wirte backend");
                    }
                    None => {
                        if rc < 0 {
                            panic!("negtive count of trie item");
                        }
                        let encoded = bincode::serialize(&Payload::new(rc as u32, value)).unwrap();
                        let mut backend = self.backend.write().unwrap();
                        backend.put(key.as_bytes(), &encoded).expect("write backend");
                    }
                };
            }
        }
    }
}

impl AsHashDB<Blake2Hasher, DBValue> for CachingDB {
    fn as_hash_db(&self) -> &dyn HashDB<Blake2Hasher, DBValue> {
        self
    }

    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<Blake2Hasher, DBValue> {
        self
    }
}

impl HashDB<Blake2Hasher, DBValue> for CachingDB {

    fn get(&self, key: &Hash, prefix: Prefix) -> Option<DBValue> {
        let k = self.cached.raw(key, prefix);
        let memrc = {
            if let Some((d, rc)) = k {
                if rc > 0 { return Some(d.clone()); }
                rc
            } else {
                0
            }
        };

        match self.payload(key) {
            Some(x) => {
                if x.count as i32 + memrc > 0 {
                    Some(x.value)
                }
                else {
                    None
                }
            }
            _ => None,
        }
    }

    fn contains(&self, key: &Hash, prefix: Prefix) -> bool {
        let k = self.cached.raw(key, prefix);
        match k {
            Some((_, rc)) if rc > 0 => true,
            _ => {
                let memrc = k.map_or(0, |(_, rc)| rc);
                match self.payload(key) {
                    Some(x) => {
                        x.count as i32 + memrc > 0
                    }
                    _ => false,
                }
            }
        }
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> Hash {
        self.cached.insert(prefix, value)
    }

    fn emplace(&mut self, key: Hash, prefix: Prefix, value: DBValue) {
        self.cached.emplace(key, prefix, value);
    }

    fn remove(&mut self, key: &Hash, prefix: Prefix) {
        self.cached.remove(key, prefix);
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Payload {
    count: u32,
    value: DBValue,
}

impl Payload {
    fn new(count: u32, value: DBValue) -> Self {
        Payload {
            count,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use std::cell::{RefCell};
    use map_store::{MemoryKV, KVDB};
    use hash_db::{EMPTY_PREFIX, HashDB};
    use super::{CachingDB};

    #[test]
    fn test_caching_insert() {
        let backend: Arc<RwLock<dyn KVDB>> = Arc::new(RwLock::new(MemoryKV::new()));

        {
            let mut db = CachingDB::new(Arc::clone(&backend));
            let foo = db.insert(EMPTY_PREFIX, b"foo");
            assert!(db.contains(&foo, EMPTY_PREFIX));

            let replicated = CachingDB::new(Arc::clone(&backend));
            assert!(!replicated.contains(&foo, EMPTY_PREFIX));
        }

        {
            // commit changes to backend db
            let mut db = CachingDB::new(Arc::clone(&backend));
            let foo = db.insert(EMPTY_PREFIX, b"foo");
            db.commit();
            assert!(db.contains(&foo, EMPTY_PREFIX));

            let replicated = CachingDB::new(Arc::clone(&backend));
            assert!(replicated.contains(&foo, EMPTY_PREFIX));
        }
    }
}
