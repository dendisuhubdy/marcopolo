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

use std::collections::HashMap;

use map_consensus::ConsensusErrorKind;
use map_core::balance::Balance;
use map_core::block::{self, Block, BlockProof, VerificationItem};
use map_core::staking::Staking;
use crate::types::{seed_info, seed_open, HolderItem, LockItem, P256PK};
use ed25519::{privkey::PrivKey, pubkey::Pubkey, signature::SignatureInfo};
use errors::{Error, ErrorKind};

#[derive(Debug, Clone)]
pub struct EpochItem {
    seed: u64,
    validators: Vec<HolderItem>,
}

pub struct APOS {
    epochInfos: HashMap<u64, EpochItem>,
    lInfo: LockItem,
    eid: u64, // current epoch id
    be_a_holdler: bool,
    lindex: i32, // current index in holder list on the epoch id
    // my_seed:        Option<seed_info>,
    seed_next_epoch: u64,
}

impl APOS {
    pub fn new() -> Self {
        APOS {
            epochInfos: HashMap::default(),
            lInfo: LockItem::default(),
            eid: 0,
            be_a_holdler: false,
            lindex: 0,
            // my_seed:           None,
            seed_next_epoch: 0,
        }
    }
    pub fn new2(info: LockItem) -> Self {
        APOS {
            epochInfos: HashMap::default(),
            lInfo: info,
            eid: 0,
            be_a_holdler: false,
            lindex: 0,
            // my_seed:       None,
            seed_next_epoch: 0,
        }
    }
    pub fn be_a_holder(&mut self, b: bool) {
        self.be_a_holdler = true;
    }
    // pub fn from_genesis(&mut self,genesis: &Block,state: &Balance) {
    //     let proofs = genesis.get_proofs();
    //     let mut vals: Vec<HolderItem> = Vec::new();
    //     let seed: u64 = 0;
    //     for (i,proof) in proofs.iter().enumerate() {
    //         if self.lInfo.equal_pk_by_slice(&proof.0[..]) {
    //             self.lindex = i as i32;
    //         }
    //         vals.push(HolderItem{
    //             pubkey:         proof.0,
    //             stakeAmount:    state.balance(proof.to_address()),
    //             sid:            -1 as i32,
    //             // seedVerifyPk:   P256PK::default(),
    //             // seedPk:         None,
    //             validator:      true,
    //         });
    //     }
    //     self.epochInfos.insert(0,EpochItem{
    //         seed:       seed,
    //         validators: vals,
    //     });
    // }

    pub fn genesis_epoch(&self, genesis: &Block, state: &Staking) -> Option<EpochItem> {
        let validators = state.validator_set();
        let mut holders: Vec<HolderItem> = Vec::new();

        for v in validators.iter() {
            let mut pk: [u8; 32] = [0; 32];
            pk.copy_from_slice(&v.pubkey);

            holders.push(HolderItem {
                pubkey: pk,
                stakeAmount: 0,
                sid: 0,
                validator: false,
            });
        }
        Some(EpochItem {
            seed: 0,
            validators: holders,
        })
    }

    pub fn next_epoch(&mut self) {
        self.eid = self.eid + 1
    }
    pub fn get_epoch_info(&self, eid: u64) -> Option<EpochItem> {
        match self.epochInfos.get(&eid) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    pub fn get_staking_holder(&self, index: u64, eid: u64) -> Option<HolderItem> {
        match self.get_epoch_info(eid) {
            Some(items) => {
                if items.validators.len() > index as usize {
                    Some(items.validators[index as usize].clone())
                } else {
                    None
                }
            }
            None => None,
        }
    }
    pub fn get_staking_holders(&self, eid: u64) -> Option<Vec<HolderItem>> {
        match self.get_epoch_info(eid) {
            Some(items) => {
                let mut vv: Vec<HolderItem> = Vec::new();
                for v in items.validators.iter() {
                    if v.is_validator() {
                        vv.push(v.clone());
                    }
                }
                if vv.len() > 0 {
                    Some(vv)
                } else {
                    None
                }
            }
            None => None,
        }
    }
    pub fn get_seed_by_epochid(&self, eid: u64) -> u64 {
        if let Some(items) = self.get_epoch_info(eid) {
            items.seed
        } else {
            0
        }
    }
    // pub fn get_seed_puk_from_validator(&self) -> Option<Vec<P256PK>> {
    //     let mut vv = Vec::new();
    //     match self.get_staking_holders(self.eid) {
    //         Some(validators)  => {
    //             for (i,v) in validators.iter().enumerate() {
    //                 vv.push(v.get_seed_puk());
    //             }
    //             Some(vv)
    //         },
    //         None    => None,
    //     }
    // }
    pub fn get_my_pos(&self) -> i32 {
        self.lindex
    }
    pub fn is_validator(&self) -> bool {
        match self.get_staking_holders(self.eid) {
            Some(vv) => match vv.get(self.lindex as usize) {
                Some(v) => {
                    return self.lInfo.equal_pk(&v.get_pubkey());
                }
                None => false,
            },
            None => false,
        }
    }
    // pub fn set_self_seed(&mut self,s: Option<seed_info>) {
    //     self.my_seed = s
    // }
    // pub fn get_self_seed(&self) -> Option<seed_info> {
    //     // self.my_seed
    //     None
    // }
    pub fn set_seed_next_epoch(&mut self, seed: u64) {
        self.seed_next_epoch = seed
    }
    pub fn get_seed_next_epoch(&self) -> u64 {
        self.seed_next_epoch
    }
    pub fn make_rand_seed(&self) -> Result<seed_info, Error> {
        // let escrow = pvss::simple::escrow(super::os_seed_share_count);
        // let msg = pvss::crypto::PublicKey{point:escrow.secret}.to_bytes();
        // let mut pubs : Vec<pvss::crypto::PublicKey> = Vec::new();
        // match self.get_seed_puk_from_validator() {
        //     Some(vv)  => {
        //         for v in vv.iter() { pubs.push(v.into()); }
        //         let commitments = pvss::simple::commitments(&escrow);
        //         let shares = pvss::simple::create_shares(&escrow, &pubs);
        //         let de: Vec<pvss::simple::DecryptedShare> = Vec::new();
        //         // verify shares
        //         for share in shares {
        //             let idx = (share.id - 1) as usize;
        //             let verified_encrypted =
        //                 share.verify(share.id, &pubs[idx], &escrow.extra_generator, &commitments);
        //             if !verified_encrypted {
        //                 println!(
        //                     "encrypted share {id}: {verified}",
        //                     id = share.id,
        //                     verified = verified_encrypted
        //                 );
        //                 return Err(ConsensusErrorKind::NoValidatorsInEpoch.into());
        //             }
        //             let b = msg.as_slice();
        //             let a: u8 = b[0];

        //             return Ok(seed_info::new(
        //                 self.get_my_pos(),
        //                 self.eid,
        //                 self.lInfo.get_my_id(),
        //                 P256PK::new(a,b[1..]),
        //                 &shares,
        //                 &de
        //             ))
        //         }
        //     },
        //     None    => return Err(ConsensusErrorKind::NoValidatorsInEpoch.into()),
        // };

        return Err(ConsensusErrorKind::NoValidatorsInEpoch.into());
    }
    pub fn recover_seed_from_shared_msg(&self, si: &seed_info) -> Result<Vec<u8>, Error> {
        // if si.index != self.lindex {
        //     return Err(ConsensusErrorKind::NotMatchLocalHolders.into());
        // }
        // if si.decrypted.len() < super::os_seed_share_count {
        //     return Err(ConsensusErrorKind::NotEnoughShares.into());
        // }
        // match pvss::simple::recover(super::os_seed_share_count, si.decrypted.as_slice()) {
        //     Ok(recovered) => { Ok(recovered.secret.to_bytes()) },
        //     Err(()) => Err(ConsensusErrorKind::RecoverSharesError.into()),
        //  }

        return Err(ConsensusErrorKind::RecoverSharesError.into());
    }
    pub fn recove_the_share(&self, si: &mut seed_info) -> Result<(), Error> {
        // for share in si.shares {
        //     if self.lindex == (share.id - 1) as i32 {
        //         let pk = self.lInfo.get_pk2();
        //         let d = pvss::simple::decrypt_share(&self.lInfo.into(), &pk, &share);
        //         let verified_decrypted = d.verify(&pk, &share);
        //         println!(
        //             "decrypted share {id}: {verified}",
        //             id = share.id,
        //             verified = verified_decrypted
        //         );
        //         if verified_decrypted {
        //             si.decrypted.push(d);
        //             return Ok(());
        //         } else {
        //             return Err(Err(ConsensusErrorKind::DecryptShareMsgError.into()))
        //         }
        //     }
        // }
        return Err(ConsensusErrorKind::NoValidatorsInEpoch.into());
    }
    // pub fn recover_share_from_seed_info(&self,si: &send_seed_info) -> Result<pvss::simple::DecryptedShare,Error> {
    //     for share in si.shares {
    //         if self.lindex == (share.id - 1) as i32 {
    //             let pk = self.lInfo.get_pk2();
    //             let d = pvss::simple::decrypt_share(&self.lInfo.into(), &pk, &share);
    //             let verified_decrypted = d.verify(&pk, &share);
    //             println!(
    //                 "decrypted share {id}: {verified}",
    //                 id = share.id,
    //                 verified = verified_decrypted
    //             );
    //             if verified_decrypted {
    //                 return Ok(d);
    //             } else {
    //                 return Err(Err(ConsensusErrorKind::DecryptShareMsgError.into()))
    //             }
    //         }
    //     }
    //     return Err(Err(ConsensusErrorKind::NoValidatorsInEpoch.into()))
    // }
    pub fn make_seed_on_epoch() -> bool {
        false
    }
}
