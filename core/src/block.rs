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

extern crate ed25519;
extern crate hash;

use serde::{Serialize, Deserialize};
// use super::traits::{TxMsg};
use super::transaction::{Transaction};
use ed25519::{signature::SignatureInfo};
// use hash;
use bincode;


#[derive(Serialize, Deserialize)]
#[derive(Debug, Default, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn to_slice(&self) -> &[u8] {
        return &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}


/// Block header
#[derive(Serialize, Deserialize, Debug)]
#[derive(Copy, Clone)]
pub struct Header {
	pub height: u64,
	pub parent_hash: Hash,
    pub time: u64,
}

impl Default for Header {
	fn default() -> Self {
		Header {
			height: 0,
			parent_hash: Hash([0; 32]),
			time: 0,
		}
	}
}

impl Header {
    pub fn hash(&self) -> Hash {
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        Hash(hash::blake2b_256(encoded))
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Default, Debug)]
pub struct VerificationItem {
    pub msg:    Hash,
    pub signs:  SignatureInfo,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Default,Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct BlockProof(pub [u8;32],pub [u8;32],pub u8);

impl BlockProof {
    pub fn new(t: u8,pk: &[u8]) -> Self {
        if t == 0u8 {
            let mut o1 = [0u8;32];
            o1[..].copy_from_slice(&pk[0..32]);
            BlockProof(o1,[0u8;32],t)
        } else {
            let mut o1 = [0u8;32];
            let mut o2 = [0u8;32];
            o1[..].copy_from_slice(&pk[0..32]);
            o2[..].copy_from_slice(&pk[32..64]);
            BlockProof(o1,o2,t)
        }
    }
    pub fn get_pk(&self,mut pk: [u8;64]) -> u8 {
        if self.2 == 0u8 {
            pk[0..32].copy_from_slice(&self.0[..]);
        } else {
            pk[0..32].copy_from_slice(&self.0[..]);
            pk[32..64].copy_from_slice(&self.1[..]);
        }
        self.2
    }
}


#[derive(Debug,Clone,Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub signs: Vec<VerificationItem>,
    pub txs:  Vec<Transaction>,
    pub proofs: Vec<BlockProof>,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            header: Default::default(),
            signs:  Vec::new(),
            txs:    Vec::new(),
            proofs: Vec::new(),
        }
    }
}

impl  Block {
    // fn new(header: Header,signs: Vec<VerificationItem>,proofs: Vec<BlockProof>) -> Self {
    //     Block{header,signs,proofs}
    // }
    fn new(header: Header,txs: Vec<Transaction>,signs: Vec<VerificationItem>,proofs: Vec<BlockProof>) -> Self {
        Block{header,signs,txs,proofs}
    }
    fn header(&self) -> &Header {
		&self.header
    }
    pub fn get_hash(&self) -> Option<Hash> {
        let code = bincode::serialize(&self).unwrap();
        let mut hh = [0u8; 32];
        hh.copy_from_slice(&code[..]);
        let mut hash_data = hash::inner_blake2b_256(hh);
        Some(Hash(hash_data))
    }
    pub fn add_proof(&mut self,proof: BlockProof) {
        self.proofs.push(proof);
    }
    pub fn proof_one(&self) -> Option<&BlockProof> {
        self.proofs.get(0)
    }
    pub fn sign_one(&self) ->Option<&VerificationItem> {
        self.signs.get(0)
    }
}

pub fn is_equal_hash(hash1: Option<Hash>,hash2: Option<Hash>) -> bool {
    hash1.map_or(false,|v|{hash2.map_or(false,|v2|{ if v == v2 {return true;} else {return false;}})})
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::bincode;

    #[test]
    fn test_header_hash() {
        let head: Header = Default::default();
        let encoded: Vec<u8> = bincode::serialize(&head).unwrap();
        assert_eq!(encoded, vec![0; 48]);
    }
}
