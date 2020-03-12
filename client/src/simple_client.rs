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

use core::block::{Block,BlockProof,VerificationItem};
use core::genesis::{ed_genesis_priv_key,ed_genesis_pub_key};
use consensus::{Error,poa::POA};
use std::thread;
use std::panic;
use std::fmt;
use std::time::{Duration, Instant};


#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct simple_client {
    pub running: bool,
}

impl simple_client {
    pub fn new_client() -> Self {
        simple_client{running:false}
    }
    pub fn start(mut self) -> bool {
        thread::spawn(move || {
            self.running = true;
            loop {
                let res = self.insert_block(self.generate_block());
                match res {
                    Ok(()) => println!("insert a block"),
                    Err(e) => println!("Error: {}", e),
                };
                thread::sleep(Duration::from_millis(POA::get_interval()));
                if !self.running {
                    break;
                }
            }
        });
        true
    }
    pub fn stop(&mut self) -> bool {
        self.running = false;
        true
    }
    pub fn new_empty_block() -> Block {
        Block::default()
    }
    pub fn generate_block(&self) -> Block {
        // 1. get txs from txpool
        // 2. exc txs
        // 3. get pre_block info
        // 4. finalize block
        simple_client::new_empty_block()
    }
    pub fn insert_block(&self,b: Block) -> Result<(),Error> {
        Ok(())
    }
}
 