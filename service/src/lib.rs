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



extern crate core;
extern crate consensus;
extern crate chain;

use core::block::{self,Block,BlockProof,VerificationItem,Header,Hash};
use core::genesis::{ed_genesis_priv_key,ed_genesis_pub_key};
use consensus::{poa::POA,Error};
use chain::blockchain::{BlockChain,};
use std::thread;
use std::panic;
use std::fmt;
use std::time::{Duration, Instant, SystemTime};

// pub mod Service;

//#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Service {
    pub running: bool,
    pub block_chain: BlockChain,
}

impl Service {
    pub fn new_service() -> Self {
        Service{
            running:        false,
            block_chain:    BlockChain::new(),
        }
    }
    pub fn start(mut self) -> bool {
        self.block_chain.load();
        let builder = thread::spawn(move || {
            self.running = true;
            loop {
                let res2 = self.generate_block();
                match res2 {
                    Ok(b) => {
                        let res = self.block_chain.insert_block(b);
                        match res {
                            Ok(()) => println!("insert a block"),
                            Err(e) => println!("insert_block Error: {:?}", e),
                        };
                    },
                    Err(e) => println!("generate_block,Error: {:?}", e),
                };  
                thread::sleep(Duration::from_millis(POA::get_interval()));
                if !self.running {
                    break;
                }
            }
        });
        builder.join();
        true
    }
    pub fn stop(&mut self) -> bool {
        self.running = false;
        true
    }
    pub fn new_empty_block() -> Block {
        Block::default()
    }
    pub fn generate_block(&mut self) -> Result<Block,Error> {
        // 1. get txs from txpool
        // 2. exc txs
        // 3. get pre_block info
        // 4. finalize block
        let cur_block = self.block_chain.current_block();
        let txs = Vec::new();
        let txs_root = block::get_hash_from_txs(txs.clone());
        let header: Header = Header{
            height: cur_block.height() + 1,
            parent_hash: cur_block.get_hash().clone(),
            tx_root:    txs_root,
            sign_root:  Hash([0;32]),
			time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
        };
        let mut b = Block::new(header,txs,Vec::new(),Vec::new());
        let finalize = POA{};
        finalize.finalize_block(b)
    }
    pub fn get_current_block(&mut self) -> Block {
        self.block_chain.current_block()
    }
    pub fn get_current_height(&mut self) -> u64 {
        self.block_chain.current_block().height()
    }
    pub fn get_block_by_height(&self,height: u64) -> Option<Block> {
        self.block_chain.get_block_by_number(height)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[test]
    fn test_service() {
        println!("begin service,for 60 seconds");
        let service = Service::new_service();
        service.start();
        thread::sleep(Duration::from_millis(60*1000));
        println!("end service");
    }
}