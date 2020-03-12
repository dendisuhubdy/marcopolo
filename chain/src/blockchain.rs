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

use crate::store::ChainDB;
use map_store;
use map_core::block::{Block, Header, Hash};
use map_core::genesis;
use map_consensus::poa;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    UnknownAncestor,
    KnownBlock,
    MismatchHash,
    InvalidBlockProof,
    InvalidBlockTime,
    InvalidBlockHeight,
    InvalidAuthority,
}

pub struct BlockChain {
    db: ChainDB,
    validator: Validator,
    genesis: Block,
    consensus: poa::POA
}

impl BlockChain {
    pub fn new() -> Self {
        let db_cfg = map_store::config::Config::default();

        BlockChain {
            db: ChainDB::new(db_cfg).unwrap(),
            genesis: genesis::to_genesis(),
            validator: Validator{},
            consensus: poa::POA{},
        }
    }

    pub fn setup_genesis(&mut self) -> Hash {
        if self.db.get_block_by_number(0).is_none() {
            self.db.write_block(&self.genesis).expect("can not write block");
        }

        self.genesis.hash()
    }

    pub fn load(&mut self) {
        if self.db.get_block_by_number(0).is_none() {
            self.setup_genesis();
        }
    }

    pub fn genesis_hash(&mut self) -> Hash {
        self.genesis.hash()
    }

    pub fn current_block(&mut self) -> Block {
        self.db.head_block().unwrap()
    }

    #[allow(unused_variables)]
    pub fn exits_block(&self, h: Hash, num: u64) -> bool {
        self.db.get_block_by_number(num).is_some()
    }

    pub fn check_previous(&self, header: &Header) -> bool {
        self.db.get_block(&header.parent_hash).is_some()
    }

    pub fn get_block_by_number(&self, num: u64) -> Option<Block> {
        self.db.get_block_by_number(num)
    }

    pub fn get_block(&self, hash: Hash) -> Option<Block> {
        self.db.get_block(&hash)
    }

    pub fn get_header_by_number(&self, num: u64) -> Option<Header> {
        self.db.get_header_by_number(num)
    }

    pub fn insert_block(&mut self, block: Block) -> Result<(), Error> {
        // Already in chain
        if self.exits_block(block.hash(), block.height()) {
            return Err(Error::KnownBlock)
        }

        if !self.check_previous(&block.header) {
            return Err(Error::UnknownAncestor)
        }

        let current = self.current_block();
        // No valid ancestor
        if block.hash() != current.hash() {
            return Err(Error::UnknownAncestor)
        }

        self.validator.validate_header(self, &block.header)?;
        if self.consensus.verify(&block).is_err() {
            return Err(Error::InvalidAuthority)
        }
        self.validator.validate_block(self, &block)?;

        self.db.write_block(&block).expect("can not write block");
        self.db.write_head_hash(block.header.hash()).expect("can not wirte head");
        Ok(())
    }
}

pub struct Validator;

impl Validator {
    #[allow(unused_variables)]
    pub fn validate_block(&self, chain: &BlockChain, block: &Block) -> Result<(), Error> {
        Ok(())
    }

    pub fn validate_header(&self, chain: &BlockChain, header: &Header) -> Result<(), Error> {
        // Ensure block parent exists on chain
        let pre = match chain.get_block(header.parent_hash) {
            Some(b) => b,
            None => return Err(Error::UnknownAncestor),
        };

        // Ensure block height increase by one
        if header.height != pre.header.height + 1 {
            return Err(Error::InvalidBlockHeight);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let mut chain = BlockChain::new();
        chain.load();
        assert_eq!(chain.genesis.height(), 0);
        assert_eq!(chain.genesis.header.parent_hash, Hash::default());
        assert!(chain.get_block_by_number(0).is_some());
    }
}