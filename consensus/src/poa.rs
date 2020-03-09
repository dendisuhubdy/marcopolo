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
extern crate ed25519;

use super::{Error,ErrorKind};
use super::traits::IConsensus;
use core::block::{Block,BlockProof,VerificationItem};
use core::genesis::{ed_genesis_priv_key,ed_genesis_pub_key};
use ed25519::{pubkey::Pubkey};


// use std::error::Error;
const poa_Version: u32 = 1;
pub struct poa {}

impl IConsensus for poa {
    fn version() -> u32 {
        poa_Version
    }
} 

impl poa {
    pub fn finalize_block(t: u8,pk: &[u8],mut b: Block) -> Result<(),Error> {
        let proof = BlockProof::new(t,pk);
        b.add_proof(proof);
        Ok(())
    }
    pub fn verify(&self,b: Block) -> Result<(),Error> {
        let proof = b.proof_one();
        match proof {
            Some(&v) => {
                let sign_info = b.sign_one();
                match sign_info {
                    Some(&v2) => self.poa_verify(&v,&v2),
                    None => Ok(()),
                }
            },
            None => Ok(()),
        }
        //Err(ErrorKind::Verify)
    }
    fn poa_verify(&self,proof: &BlockProof,vInfo: &VerificationItem) -> Result<(),Error> {
        let mut pk0 = [0u8;64];
        let t = proof.get_pk(pk0);
        if t == 0u8 {       // ed25519
            let pk = Pubkey::from_bytes(&pk0);
            let msg = vInfo.to_msg();
            let res = pk.verify(&msg,&vInfo.signs);
            match res {
                Ok(n) => Ok(()),
                Err(e) => Err(()), 
            };
        }
        Ok(())
    }
}