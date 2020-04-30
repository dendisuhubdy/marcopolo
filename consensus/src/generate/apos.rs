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
use core::balance::{Balance};
use std::collections::HashMap;
use super::types::{ValidatorItem,LockItem,P256PK};

#[derive(Debug, Clone)]
pub struct EpochItem {
    seed: u64,
    validators: Vec<ValidatorItem>,    
}

pub struct APOS {
    epochInfos: HashMap<u64,EpochItem>,
    lInfo:  LockItem,    
    eid: u64,
}

impl APOS {
    pub fn new() -> Self {
        APOS{
            epochInfos: HashMap::default(),
            lInfo:      LockItem::default(),
            eid: 0,
        }
    }
    pub fn from_genesis(&mut self,genesis: &Block,state: &Balance) {
        let proofs = genesis.get_proofs();
        let mut vals: Vec<ValidatorItem> = Vec::new();
        let seed: u64 = 0;
        for proof in proofs {
            vals.push(ValidatorItem{
                pubkey:         proof.0,
                stakeAmount:    state.balance(proof.to_address()),
                sid:            -1 as i32,
                seedVerifyPk:   P256PK::default(),
            });
        }
        self.epochInfos.insert(0,EpochItem{
            seed:       seed,
            validators: vals,
        });
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
    pub fn get_validator(&self, index: i32,eid: u64) -> Option<ValidatorItem> {
        match self.get_epoch_info(eid) {
            Some(items)  =>{
                if items.validators.len() > index as usize{
                    Some(items.validators[index as usize].clone())
                } else {
                    None
                }
            },
            None => None,
        } 
    }
    pub fn get_validators(&self, eid: u64) -> Option<Vec<ValidatorItem>> {
        match self.get_epoch_info(eid) {
            Some(items) => {
                Some(items.validators)
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