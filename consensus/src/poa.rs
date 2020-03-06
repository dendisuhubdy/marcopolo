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

use super::traits::IConsensus;
use core::block::{Block,BlockProof};
use super::Error;
// use std::error::Error;
pub struct poa {

}

impl IConsensus for poa {
    fn version() -> u32 {
        1u32
    }
} 

impl poa {
    pub fn finalize_block(t: u8,pk: &[u8],mut b: Block) -> Result<(),Error> {
        let proof = BlockProof::new(t,pk);
        b.add_proof(proof);
        Ok(())
    }
}