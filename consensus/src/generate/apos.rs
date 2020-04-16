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

use ed25519::{pubkey::Pubkey,privkey::PrivKey,signature::SignatureInfo};
use core::block::{self,Block,BlockProof,VerificationItem};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ValidatorItem {
    pubkey: [u8;32],
    stakeAmount: u128,
}
impl From<ValidatorItem> for Pubkey {
    fn from(v: ValidatorItem) -> Self {
        Pubkey::from_bytes(&v.pubkey)
    }
}

#[derive(Debug, Clone)]
struct EpochItem {
    seed: u64,
    validators: Vec<ValidatorItem>,    
}
pub struct APOS {
    epochInfos: HashMap<u64,EpochItem>,
    eid: u64,
}

impl APOS {
    pub fn new() -> Self {
        APOS{
            epochInfos: HashMap::default(),
            eid: 0,
        }
    }
    pub fn from_genesis(&mut self,genesis: &Block,state: &Balance) {
        let &proofs = genesis.get_proofs();
        let mut vals: Vec<ValidatorItem> = Vec::new();
        for proof in proofs {
            vals.push(ValidatorItem{
                pubkey:         proof.0,
                stakeAmount:    state.Balance(proof.to_address()),
            });
        }
        self.epochInfos.insert(0,vals);
    }
    pub fn next_epoch(&mut self) {
        self.eid = self.eid + 1
    }
    pub fn get_epoch_info(&self,eid: u64) -> Option<EpochItem> {
        match self.epochInfos.get(&eid) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    pub fn get_validator(&self, index: u32,eid: u64) -> Option<ValidatorItem> {
        match self.get_epoch_info(eid) {
            Some(items)  =>{
                let i : i32 = index as i32;
                if items.validators.len() > i {
                    Some(items.validators[i].clone())
                } else {
                    None
                }
            },
            None => None,
        } 
    }
    pub fn get_validators(&self, eid: u64) -> Option<&Vec<ValidatorItem>> {
        match self.get_epoch_info(eid) {
            Some(items) => {
                Some(&items.validators)
            },
            None => None,
        }
    }
    pub fn get_seed_by_epochid(&self,eid: u64) -> u64 {
        if let Some(items) = self.get_epoch_info(eid) {
            items.seed
        } else {
            0
        }
    }
}