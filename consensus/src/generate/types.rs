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
use ed25519::{pubkey::Pubkey,privkey::PrivKey};

#[derive(Debug, Clone)]
pub struct P256PK (pub u8,pub [u8;32]);
pub type seed_open = P256PK;

impl Default for P256PK {
    fn default() -> Self {
        Self(0,[0u8;32])
    }
}
impl P256PK {
    pub fn new(a: u8,b: [u8]) -> Self {
        let mut c = [0u8;32];
        c[..].copy_from_slice(&b[..]);
        Self(a,c)
    }
    pub fn to_bytes(&self,a: &mut[u8]) {
        a[0] = self.0;
        a[1..].copy_from_slice(&self.1[..]);
    }
    pub fn from_bytes(c: &[u8]) -> Self {
        let mut b = [0u8;32];
        let a = c[0];
        b[..].copy_from_slice(&c[1..]);
        Self(a,b)
    }
}

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
pub struct HolderItem {
    pub pubkey: [u8;32],
    pub seedVerifyPk: P256PK,
    pub seedPk:       Option<P256PK>,
    pub stakeAmount: u128,
    pub sid:        i32,
    pub validator:  bool,
}
impl HolderItem {
    pub fn set_sid(&mut self,i: i32) {
        self.sid = i;
    }
    pub fn get_sid(&self) -> i32 {
        self.sid
    }
    pub fn get_seed_puk(&self) -> P256PK {
        self.seedVerifyPk
    }
    pub fn is_validator(&self) -> bool {
        self.validator
    }
}
impl From<HolderItem> for Pubkey {
    fn from(v: HolderItem) -> Self {
        Pubkey::from_bytes(&v.pubkey)
    }
}
impl From<HolderItem> for Stakeholder {
    fn from(v: HolderItem) -> Self {
        Stakeholder{
            name:   String::from_utf8_lossy(&v.pubkey[..4]).to_string(),
            coins:  v.stakeAmount,
            index:  -1 as i32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LockItem {
    key1:   [u8;32],        // for sign message
    key2:   [u8;32],        // for decrypted the seed message
}

impl LockItem {
    pub fn equal_pk(&self,pk: &[u8]) -> bool {
        let l_priv: PrivKey = self.into();
        match l_priv.to_pubkey(){
            Ok(l_pk) => {
                return l_pk.equal(Pubkey::from_bytes(&pk))
            },
            Err(e) => {
                println!("to_pubkey(in eqaul pk function) failed, Error: {:?}", e);
                false
            },
        }
    }
}
impl Default for LockItem {
    fn default() -> Self {
        Self{
            key1:   [0u8;32],
            key2:   [0u8;32],
        }
    }
}
impl From<LockItem> for PrivKey {
    fn from(v: LockItem) -> Self {
        PrivKey::from_bytes(&v.key1)
    }
}
impl From<LockItem> for pvss::crypto::PrivateKey {
    fn from(v: LockItem) -> Self {
        pvss::crypto::PrivateKey::from_bytes(&v.key2)
    }
}

pub struct seed_info {
    pub index:  i32,
    pub msg:    seed_open,
    pub shares: Vec<pvss::simple::EncryptedShare>
}
impl seed_info {
    pub fn new(i: i32,s: seed_open,shs: &Vec<pvss::simple::EncryptedShare>) -> Self {
        Self{
            index:  i,
            msg:    msg,
            shares: shs,
        }
    }
    pub fn get_commit_phase_msg(&self) -> (Hash,Vec<pvss::simple::EncryptedShare>) {
        let mut data: [u8;33] = [0u8;33];
        self.msg.to_bytes(&mut data);
        let h = Hash::make_hash(&data);
        (h,self.shares.clone())
    }
    pub fn get_Revel_phase_msg(&self) -> seed_open {
        self.msg.clone()
    }
}