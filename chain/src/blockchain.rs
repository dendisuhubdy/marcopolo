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
use map_core;
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
            self.db.write_head_hash(self.genesis.hash()).expect("can not wirte head");
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

        if block.header.parent_hash != current.hash() {
            return Err(Error::UnknownAncestor)
        }

        self.validator.validate_header(self, &block.header)?;
        self.validator.validate_block(self, &block)?;
        if let Err(e) = self.consensus.verify(&block) {
            error!("consensus err height={}, {:?}", block.height(), e);
            return Err(Error::InvalidAuthority);
        }

        self.db.write_block(&block).expect("can not write block");
        self.db.write_head_hash(block.header.hash()).expect("can not wirte head");
        info!("insert block, height={}, hash={}, previous={}", block.height(), block.hash(), block.header.parent_hash);
        Ok(())
    }
}

pub struct Validator;

impl Validator {
    #[allow(unused_variables)]
    pub fn validate_block(&self, chain: &BlockChain, block: &Block) -> Result<(), Error> {
        if block.header.sign_root != map_core::block::get_hash_from_signs(block.signs.clone()) {
            return Err(Error::MismatchHash);
        }

        if block.header.tx_root != map_core::block::get_hash_from_txs(block.txs.clone()) {
            return Err(Error::MismatchHash);
        }

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

        // Ensure block time interval
        if header.time <= pre.header.time {
            return Err(Error::InvalidBlockTime);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_init() {
        let mut chain = BlockChain::new();
        chain.load();
        assert_eq!(chain.genesis.height(), 0);
        assert_eq!(chain.genesis.header.parent_hash, Hash::default());
        assert!(chain.get_block_by_number(0).is_some());
    }

    #[test]
    fn test_insert_empty() {
        let mut chain = BlockChain::new();
        chain.load();
        {
            let block = Block {
                header: Header{
                    height: 1,
                    ..Default::default()
                },
                ..Block::default()
            };
            let ret = chain.insert_block(block);
            assert!(ret.is_err());
        }

        {
            let mut block = Block::default();
            block.header.height = 1;
            block.header.parent_hash = chain.genesis_hash();
            let ret = chain.insert_block(block);
            assert!(ret.is_err());
        }

        {
            let mut block = Block::default();
            block.header.height = 1;
            block.header.parent_hash = chain.genesis_hash();
            block.header.time = SystemTime::now().duration_since(
                SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let ret = chain.insert_block(block);
            assert!(ret.is_err());
        }
    }
}
