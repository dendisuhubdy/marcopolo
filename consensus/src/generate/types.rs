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

use core::types::{Hash,Address};
use ed25519::{pubkey::Pubkey};

#[derive(Debug, Clone)]
pub struct Stakeholder {
    pub name:   String,
    pub coins:  u128,
    pub index:  i32,
}
impl Stakeholder {
    pub fn getName(&self) -> String {
        return self.name.clone()
    }
    pub fn getCoins(&self) -> u128 {
        return self.coins
    }
    pub fn toBytes(&self) -> Vec<u8>{
        format!("{}{}",self.name,self.coins).into_bytes()
    }
    pub fn to_String(&self) -> String {
        return self.name.clone()
    }
    pub fn clone(&self) -> Self {
        return Stakeholder{
            name:	self.name.clone(),
            coins: 	self.coins,
            index:  self.index,
        }
    }
    pub fn get_index(&self) -> i32 {
        self.index
    }
    pub fn set_index(&mut self,i: i32) {
        self.index = i;
    }
}

#[derive(Debug, Clone)]
pub struct ProofEntry {
    pub hash: 	Hash,
	pub x1:		u128,
	pub x2:		u128,
}

impl ProofEntry {
    pub fn getLeftBound(&self) -> u128 {
        return self.x1
    }
    pub fn getRightBound(&self) -> u128 {
        return self.x2
    }
    pub fn getMerkleHash(&self) -> Hash {
        return self.hash
    }
    pub fn to_string(&self) -> String {
        return format!("{:?},{},{}",self.hash,self.x1,self.x2)
    }
    pub fn new_proof_entry(hash: Hash,amount1: u128,amount2: u128) -> Self {
        return ProofEntry{
            hash: 	hash,
            x1:		amount1,
            x2:		amount2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ftsResult {
    pub sholder: 	    Option<Stakeholder>,
	pub merkleProof:	Vec<ProofEntry>,
}

impl ftsResult {
    pub fn getStakeholder(&self) -> &Option<Stakeholder> {
        return &self.sholder
    }
    pub fn getMerkleProof(&self) -> &Vec<ProofEntry> {
        return &self.merkleProof
    }
    pub fn to_string(&self) -> String {
        let mut proofs: String = "".to_string();
        for v in &self.merkleProof {
            let tmp = v.to_string() + "\n";
            proofs = proofs + &tmp;
        }
        return format!("merkleProof [\n {} ]\n stakeholder \n {} \n",proofs,
        self.sholder.as_ref().unwrap().to_String())
    }
    pub fn new_fts_result(sholder: &Stakeholder,proofs: Vec<ProofEntry>) -> Self {
        return ftsResult{
            sholder: 	Some(sholder.clone()),
            merkleProof: proofs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidatorItem {
    pubkey: [u8;32],
    stakeAmount: u128,
    sid:        i32,
}
impl ValidatorItem {
    pub fn set_sid(&mut self,i: i32) {
        self.sid = i;
    }
}
impl From<ValidatorItem> for Pubkey {
    fn from(v: ValidatorItem) -> Self {
        Pubkey::from_bytes(&v.pubkey)
    }
}
impl From<ValidatorItem> for Stakeholder {
    fn from(v: ValidatorItem) -> Self {
        Stakeholder{
            name:   String::from_utf8_lossy(&v.pubkey[..4]).to_string(),
            coins:  v.stakeAmount,
            index:  -1 as i32,
        }
    }
}