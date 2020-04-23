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
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

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

#[derive(Debug, Clone,Default)]
pub struct Node {
	pub left: 		Option<Box<Node>>,
    pub right: 		Option<Box<Node>>,
	pub sholder: 	Option<Stakeholder>,
	pub hash: 		Hash,
}

impl Node {
    pub fn isLeaf(&self) -> bool {
        return self.sholder.is_some()
    }
    pub fn getStakeholder(&self) -> &Option<Stakeholder> {
        return &self.sholder
    }
    pub fn getLeftNode(&self) -> &Option<Box<Node>> {
        return &self.left
    }
    pub fn getRightNode(&self) -> &Option<Box<Node>> {
        return &self.right
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
    pub fn newNodeFromSHolder(s: Stakeholder) -> Self {
        return Node{
            left:		None,
            right: 		None,
            sholder:	Some(s.clone()),
            hash:		make_hash(&s.toBytes()),
        } 
    }
    pub fn newNode1(left: Option<Box<Node>>,right: Option<Box<Node>>,hash: Hash) -> Self {
        return Node{
            left:		left,
            right: 		right,
            sholder:	None,
            hash:		hash,
        }
    }
    pub fn newNode2(left: Node,right: Node,hash: Hash) -> Self {
        return Node{
            left:		Some(Box::new(left.clone())),
            right: 		Some(Box::new(right.clone())),
            sholder:	None,
            hash:		hash,
        }
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

pub fn makeNodeHash(left: &[u8],right: &[u8],leftValue: &[u8],rightValue: &[u8]) -> Hash {
    let mut b: Vec<u8> = Vec::new();
    b.append(&mut left.clone().to_vec());
    b.append(&mut right.clone().to_vec());
    b.append(&mut leftValue.clone().to_vec());
    b.append(&mut rightValue.clone().to_vec());
	return make_hash(b.as_slice())
}
pub fn nextInt(max: u128,rnd: &mut StdRng) -> u128 {
	return rnd.gen_range(0,max)
}

pub fn CreateMerkleTree(stakeholders: Vec<Stakeholder>) -> Vec<Box<Node>> {
    let mut tree: Vec<Box<Node>> = Vec::new();
    tree.resize(stakeholders.len() * 2,Box::new(Node::default()));
    println!("Creating Merkle tree with:{} nodes",tree.len() - 1);
    for i in 0..stakeholders.len() {
        if let Some(v) = tree.get_mut(i) {
            *v = Box::new(Node::newNodeFromSHolder(stakeholders.get(i).unwrap().clone()));
        }
    }
    for i in (1..stakeholders.len()).rev() {
        let mut left: Box<Node>;
        let mut right: Box<Node>;
        let mut h: Hash;
        {
            left = tree.get(i*2).unwrap().clone();
            right = tree.get(i*2 + 1).unwrap().clone();
            h = makeNodeHash(left.getMerkleHash().to_slice(),
                                right.getMerkleHash().to_slice(),
                                &left.getCoins().to_string().into_bytes(),
                                &right.getCoins().to_string().into_bytes());
        }
        if let Some(v) = tree.get_mut(i) {
            *v = Box::new(Node::newNode1(Some(left), Some(right), h));
        }
    }
    for i in (1..tree.len()) {
        println!("HASH:{},Index:{}",tree.get(i).unwrap().getMerkleHash(),i);
    }
	return tree;
}