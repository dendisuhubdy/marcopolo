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

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use bincode;
use hash_db::{HashDB, AsHashDB, Prefix};
use trie_db::DBValue;
use map_store::mapdb::MapDB;
use map_store::Config;
use crate::types::Hash;
use crate::trie::{MemoryDB, EMPTY_TRIE, Blake2Hasher};

#[derive(Clone)]
pub struct CachingDB {
    backend: Arc<MapDB>,
    cached: MemoryDB,
}

impl CachingDB {
    /// Create a storage backend of trie structure along with memory caching
    pub fn new(backend: Arc<MapDB>) -> Self {
        CachingDB {
            backend: backend,
            cached: MemoryDB::new(EMPTY_TRIE),
        }
    }

    fn payload(&self, key: &Hash) -> Option<Payload> {
        if let Some(data) = self.backend.get(key.as_bytes()) {
            let value: Payload = bincode::deserialize(&data.as_slice()).unwrap();
            Some(value)
        } else {
            None
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
    use std::sync::Arc;
    use map_store::Config;
    use map_store::mapdb::MapDB;
    use hash_db::{EMPTY_PREFIX, HashDB};
    use super::{CachingDB};

    #[test]
    fn test_caching_insert() {
        let mut m = MapDB::open(Config::default()).unwrap();
        let mut db = CachingDB::new(Arc::new(m));
        let foo = db.insert(EMPTY_PREFIX, b"foo");
        assert!(db.contains(&foo, EMPTY_PREFIX));
    }
}
