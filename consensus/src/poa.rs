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

use super::{Error,ErrorKind,ConsensusErrorKind};
use super::traits::IConsensus;
use core::block::{self,Block,BlockProof,VerificationItem,Hash};
use core::genesis::{ed_genesis_priv_key,ed_genesis_pub_key};
use ed25519::{pubkey::Pubkey,privkey::PrivKey,signature::SignatureInfo};


const poa_Version: u32 = 1;
pub struct POA {}

impl IConsensus for POA {
    fn version() -> u32 {
        poa_Version
    }
}

impl POA {
    pub fn sign_block(t: u8,pkey: Option<PrivKey>,mut b: Block) -> Result<(),Error> {
        let h = b.get_hash();
        match pkey {
            Some(p) => {
                if t == 0u8 {
                    let h = b.get_hash();
                    let signs = p.sign(h.to_slice());
                    POA::add_signs_to_block(h,signs,b)
                } else {
                    Ok(())
                }
            },
            None => {
                if t == 0u8 {
                    let pkey = PrivKey::from_bytes(&ed_genesis_priv_key);
                    let h = b.get_hash();
                    let signs = pkey.sign(h.to_slice());
                    POA::add_signs_to_block(h,signs,b)
                } else {
                    Ok(())
                }
            },
        }
    }
    fn add_signs_to_block(h:Hash,signs: SignatureInfo,mut b: Block) -> Result<(),Error> {
        let signs = VerificationItem::new(h,signs);
        b.add_verify_item(signs);
        let signs = b.get_signs();
        let h = block::get_hash_from_signs(signs);
        b.set_sign_hash(h);
        Ok(())
    }
    fn add_proof_to_block(t: u8,pk: &[u8],mut b: Block) -> Result<(),Error> {
        let proof = BlockProof::new(t,pk);
        b.add_proof(proof);
        Ok(())
    }
    pub fn finalize_block(&self,mut b: Block) -> Result<(),Error> {
        // sign with default priv key
        POA::sign_block(0u8,None,b)
    }
    pub fn verify(&self,b: &Block) -> Result<(),Error> {
        let proof = b.proof_one();
        match proof {
            Some(&v) => {
                let sign_info = b.sign_one();
                match sign_info {
                    Some(&v2) => self.poa_verify(&v,&v2),
                    None => Err(ConsensusErrorKind::NoneSign.into()),
                }
            },
            None => {
                // get proof from genesis
                let proof = BlockProof::new(0u8,&ed_genesis_pub_key);
                let sign_info = b.sign_one();
                match sign_info {
                    Some(&v2) => self.poa_verify(&proof,&v2),
                    None => Err(ConsensusErrorKind::NoneSign.into()),
                }
            },
        }
    }

    fn poa_verify(&self,proof: &BlockProof,vInfo: &VerificationItem) -> Result<(),Error> {
        let mut pk0 = [0u8;64];
        let t = proof.get_pk(pk0);
        if t == 0u8 {       // ed25519
            let pk = Pubkey::from_bytes(&pk0);
            let msg = vInfo.to_msg();
            let res = pk.verify(&msg,&vInfo.signs);
            match res {
                Ok(()) => Ok(()),
                Err(e) => Err(ConsensusErrorKind::Verify.into()),
            }
        } else {
            Ok(())
        }
    }
    pub fn get_interval() -> u64 {
        2000u64
    }
}