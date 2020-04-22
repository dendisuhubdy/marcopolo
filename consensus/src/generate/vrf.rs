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
    pub fn toString(&self) -> String {
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
