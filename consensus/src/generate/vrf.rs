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
use std::sync::Arc;
use std::cmp::Ordering;

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
	pub left: 		Option<Arc<Node>>,
    pub right: 		Option<Arc<Node>>,
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
    pub fn getLeftNode(&self) -> &Option<Arc<Node>> {
        return &self.left
    }
    pub fn getRightNode(&self) -> &Option<Arc<Node>> {
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
    pub fn newNode1(left: Option<Arc<Node>>,right: Option<Arc<Node>>,hash: Hash) -> Self {
        return Node{
            left:		left,
            right: 		right,
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

pub fn CreateMerkleTree(stakeholders: Vec<Stakeholder>) -> Vec<Arc<Node>> {
    let mut tree: Vec<Arc<Node>> = Vec::new();
    tree.resize(stakeholders.len() * 2,Arc::new(Node::default()));
    println!("Creating Merkle tree with:{} nodes",tree.len() - 1);
    for i in 0..stakeholders.len() {
        if let Some(v) = tree.get_mut(i) {
            *v = Arc::new(Node::newNodeFromSHolder(stakeholders.get(i).unwrap().clone()));
        }
    }
    for i in (1..stakeholders.len()).rev() {
        let mut left: Arc<Node>;
        let mut right: Arc<Node>;
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
            *v = Arc::new(Node::newNode1(Some(left), Some(right), h));
        }
    }
    for i in (1..tree.len()) {
        println!("HASH:{},Index:{}",tree.get(i).unwrap().getMerkleHash(),i);
    }
	return tree;
}
pub fn FtsTree(tree: Vec<Arc<Node>>,rnd: &mut StdRng) -> Box<ftsResult> {
    let mut merkleProof: Vec<ProofEntry> = Vec::new();
	let mut i: usize = 1;
	loop {
		if tree[i].isLeaf() {
            let s = tree[i].getStakeholder();
			return Box::new(ftsResult::newFtsResult(s.as_ref().unwrap(),merkleProof));
        }
        let x1 = tree.get(i).unwrap()
                    .getLeftNode()
                    .as_ref()
                    .unwrap()
                    .getCoins();
        let x2 = tree.get(i).unwrap()
                    .getRightNode()
                    .as_ref()
                    .unwrap()
                    .getCoins();
		println!("left subtree coins:{} right subtree coins:{}",x1,x2);
		let r = nextInt(x1 + x2,rnd) + 1;
		println!("Picking coin number:{}",r);
		if r <= x1 {
			println!("Choosing left subtree...");
            i *= 2;
            merkleProof.push(ProofEntry::newProofEntry(
                tree.get(i+1).unwrap().getMerkleHash(), 
                x1, x2));
		} else {
			println!("Choosing right subtree...");
            i = 2*i + 1;
            merkleProof.push(ProofEntry::newProofEntry(
                tree.get(i-1).unwrap().getMerkleHash(), 
                x1, x2));
		}
	}
}
pub fn FtsVerify(merkleRootHash: Hash, res: Box<ftsResult>,rnd: &mut StdRng) -> bool {
    let mut resPath: Vec<u8> = Vec::new(); 
	for v in res.getMerkleProof().iter() {
        let x1 = v.getLeftBound();
        let x2 = v.getRightBound();
		let r = nextInt(x1 + x2,rnd) + 1;
		if r <= x1 {
            println!("0");
            resPath.push(0);
		} else {
			println!("1");
			resPath.push(1);
		}
	}
    println!("OK");
    let ss = res.getStakeholder();
    let mut hx = make_hash(&ss.as_ref().unwrap().toBytes());
    for i in (0..res.getMerkleProof().len()).rev() {
        let proof = res.getMerkleProof().get(i).unwrap();
        let x1 = proof.getLeftBound().to_string().into_bytes();
        let x2 = proof.getRightBound().to_string().into_bytes();
		let hy = proof.getMerkleHash();
		if resPath[i] == 0_u8 {
			hx = makeNodeHash(hx.to_slice(),hy.to_slice(),&x1,&x2)
		} else {
			hx = makeNodeHash(hy.to_slice(),hx.to_slice(),&x1,&x2)
		}
		println!("Next hash:{}",hx);
    }
    if Ordering::Equal == merkleRootHash.to_vec().cmp(&hx.clone().to_vec()) {
        println!("Root hash matches!");
        return true
    } else {
        println!("Invalid Merkle proof");
        return true
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    #[test]
    fn testBox01() {
        let mut bx = Box::new(5_i32);
        let mut bx_new = &bx;
        let mut bx_new_clone = bx_new.clone();
        *bx_new_clone = 8;
        // println!("bx <value> address  : {:p}", &*bx);//box中5_f32的地址
        // println!("bx address          : {:p}", &bx);//指针的指针
        // println!("bx_new address      : {:p}", &bx_new);
        // println!("bx_new_clone address: {:p}", &*bx_new_clone);
        println!("bx:{}",bx);
        println!("bx:{}",bx_new_clone);
    }
}
