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

use map_store::mapdb::MapDB;
use map_store::config::Config;
use map_store::Error;
use map_core::block::{Header, Hash};
// use map_core::block;
use bincode;

const HEADER_PREFIX: u8 = 'h' as u8;
const HEAD_PREFIX: u8 = 'H' as u8;
const BLOCK_PREFIX: u8 = 'b' as u8;

const HEADERHASH_PREFIX: u8 = 'n' as u8;


/// Blockchain storage backend implement
pub struct ChainDB {
    db: MapDB,
}

impl ChainDB {

    pub fn new() -> Result<Self, Error> {
        let cfg = Config::default();
        let m = MapDB::open(cfg).unwrap();

        Ok(ChainDB{db: m})
    }

    pub fn write_header(&mut self, h: &Header) -> Result<(), Error> {
        let encoded: Vec<u8> = bincode::serialize(h).unwrap();
        let key = Self::header_key(&(h.hash().0));
        self.write_header_hash(h.height, &h.hash());
        self.db.put(&key, &encoded)
    }

    pub fn read_header(&mut self, num: u64) -> Option<Header> {
        let header_hash = match self.read_header_hash(num) {
            Some(h) => h,
            None => return None,
        };
        let key = Self::header_key(&(header_hash.0));
        let serialized = match self.db.get(&key.as_slice()) {
            Some(s) => s,
            None => return None,
        };

        let header: Header = bincode::deserialize(&serialized.as_slice()).unwrap();
        Some(header)
    }

    pub fn read_header_hash(&mut self, num: u64) -> Option<Hash> {
        let key = Self::header_hash_key(num);
        self.db.get(&key).map(|h| {
            let mut hash: Hash = Default::default();
            hash.0.copy_from_slice(h.as_slice());
            hash
        })
    }

    pub fn write_header_hash(&mut self, num: u64, hash: &Hash) -> Result<(), Error> {
        let key = Self::header_hash_key(num);
        self.db.put(&key, hash.to_slice())
    }

    fn header_key(_hash: &[u8]) -> Vec<u8> {
        let mut pre = Vec::new();
        pre.push(HEADER_PREFIX);
        pre.extend_from_slice(_hash);
        pre
    }

    fn header_hash_key(num: u64) -> Vec<u8> {
        let mut pre = Vec::new();
        pre.push(HEADERHASH_PREFIX);
        let bytes = num.to_be_bytes();
        pre.extend_from_slice(&bytes);
        pre
    }
}
