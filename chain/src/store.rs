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
use core::block::Header;
use bincode;

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
        let m = MapDB::open(cfg).unwrap();

        Ok(ChainDB{db: m})
    }

    pub fn write_header(&mut self, h: &Header) -> Result<(), Error> {
        let encoded: Vec<u8> = bincode::serialize(h).unwrap();
        let key = Self::header_key(HEADER_PREFIX, &(h.hash().0));
        self.db.put(&key, &encoded)
    }

    fn header_key(prefix: u8, _hash: &[u8]) -> Vec<u8> {
        let mut pre = Vec::new();
        pre.push(prefix);
        pre.extend_from_slice(_hash);
        pre
    }
}
