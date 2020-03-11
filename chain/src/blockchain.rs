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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    UnknownAncestor,
    KnownBlock,
    MismatchHash,
}

pub struct BlockChain {
    db: ChainDB,
    validator: Validator,
    genesis: Block,
}

impl BlockChain {
    pub fn new() -> Self {
        let db_cfg = map_store::config::Config::default();

        BlockChain {
            db: ChainDB::new(db_cfg).unwrap(),
            genesis: genesis::to_genesis(),
            validator: Validator{},
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

        self.validator.validate_block(&block)?;

        self.db.write_block(&block).expect("can not write block");
        self.db.write_head_hash(block.header.hash()).expect("can not wirte head");
        Ok(())
    }
}

pub struct Validator;
// pub struct Validator<'a> {
//     chain: &'a BlockChain,
// }


impl Validator {
    #[allow(unused_variables)]
    pub fn validate_block(&self, block: &Block) -> Result<(), Error> {
        Ok(())
    }
}