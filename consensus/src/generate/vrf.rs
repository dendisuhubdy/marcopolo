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

pub fn make_hash(data: &[u8]) -> Hash {
    Hash::make_hash(data)
}

#[derive(Debug, Clone)]
pub struct Stakeholder {
    pub name:   String,
    pub coins:  u128,
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node<'a> {
	pub left: 		Option<&'a Node<'a>>,
    pub right: 		Option<&'a Node<'a>>,
	pub sholder: 	Option<Stakeholder>,
	pub hash: 		Hash,
}

impl<'a> Node<'a> {
    pub fn isLeaf(&self) -> bool {
        return self.sholder.is_some()
    }
    pub fn getStakeholder(&self) -> &Option<Stakeholder> {
        return &self.sholder
    }
    pub fn getLeftNode(&self) -> Option<&'a Node<'a>> {
        return self.left
    }
    pub fn getRightNode(&self) -> Option<&'a Node<'a>> {
        return self.right
    }
    pub fn getMerkleHash(&self) -> Hash {
        return self.hash.clone()
    }
    pub fn getCoins(&self) -> u128 {
        if self.isLeaf() {
            self.sholder.as_ref().unwrap().getCoins()
        } else {
            return self.left.as_ref().unwrap().getCoins() + self.right.as_ref().unwrap().getCoins()
        }
    }
    pub fn newNodeFromSHolder(s: &Stakeholder) -> Option<Node> {
        return Some(Node{
            left:		None,
            right: 		None,
            sholder:	Some(s.clone()),
            hash:		make_hash(&s.toBytes()),
        }) 
    }
    pub fn newNode1(left: &'a Node,right: &'a Node,hash: Hash) -> Option<Node<'a>> {
        return Some(Node{
            left:		Some(left),
            right: 		Some(right),
            sholder:	None,
            hash:		hash,
        })
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
    pub fn toString(&self) -> String {
        return format!("{},{},{}",self.hash,self.x1,self.x2)
    }
    pub fn newProofEntry(hash: Hash,amount1: u128,amount2: u128) -> Self {
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
    pub fn toString(&self) -> String {
        // let mut proofs: Vec<String> = Vec::new();
        let mut proofs: String = "".to_string();
        for v in &self.merkleProof {
            let tmp = v.toString() + "\n";
            proofs = proofs + &tmp;
        }
        return format!("merkleProof [\n {} ]\n stakeholder \n {} \n",proofs,
        self.sholder.as_ref().unwrap().to_String())
    }
    pub fn newFtsResult(sholder: &Stakeholder,proofs: Vec<ProofEntry>) -> Self {
        return ftsResult{
            sholder: 	Some(sholder.clone()),
            merkleProof: proofs,
        }
    }
}