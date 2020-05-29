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

use std::{thread,time::Duration};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::convert::TryInto;

use ed25519::{pubkey::Pubkey,privkey::PrivKey,signature::SignatureInfo};
use map_core::block::{self,Block,BlockProof,VerificationItem};
use crossbeam_channel::{bounded, select, Receiver, RecvError, Sender};
use map_core::types::Hash;
use errors::{Error, ErrorKind};
use super::types::{seed_info, HolderItem};
use super::{apos::APOS,types};
// use super::fts;
use super::ConsensusErrorKind;


const epoch_length: i32 = 100;
type TypeNewBlockEvent = Receiver<Block>;
type TypeNewTimerIntervalEvent = Receiver<()>;
pub type TypeStopEpoch = Sender<()>;

// block header has the pair of the (sid,height)
pub struct tmp_blocks {

}
impl tmp_blocks {
    pub fn make_new_block(&self,height: u64,h: Hash) -> Option<Block> {
        Some(Block::default())
    }
    pub fn get_current_Height(&self) -> u64 {
        0
    }
    pub fn get_hash_by_height(&self,height: u64) -> Option<Hash> {
        Some(Hash([0u8;32]))
    }
    pub fn get_sid_from_current_block(&self) -> i32 {
        0
    }
    pub fn get_best_chain(&self,height: u64) -> Option<Block> {
        Some(Block::default())
    }
    pub fn make_seed_in_epoch(&self,eid: u64) -> u64 {
        let (low,hi) = Epoch::get_height_from_eid(eid);
        for i in low..hi {
            let b = self.get_best_chain(i);
        }
        0
    }
}

pub struct Epoch {}

impl Epoch {
    pub fn get_epoch_from_height(h: u64) -> u64 {
        let eid: u64 = h / epoch_length as u64 + 1;
        eid
    }
    pub fn get_epoch_from_id(sid: i32, cur_eid: u64) -> u64 {
        let mut eid = cur_eid;
        if sid >= epoch_length {
            eid = cur_eid + 1
        }
        eid
    }
    pub fn get_height_from_eid(eid: u64) ->(u64,u64) {
        if eid as i64 <= 0 {
            return (0,0);
        }
        let low: u64 = (eid -1) *  epoch_length as u64;
        let hi: u64 = eid * epoch_length as u64 - 1;
        (low,hi)
    }
}

#[derive(Debug, Clone)]
pub struct Slot {
    timeout:    u32,     // millsecond
    id:         i32,
    vindex:     u32,
}
impl Slot {
    pub fn new(sid: i32,index: u32) -> Self {
        Slot{
            timeout:    5000,
            id:         sid,
            vindex:     index,
        }
    }
}
pub struct EpochProcess {
    myid:           Pubkey,
    cur_eid:        u64,
    cur_seed:       u64,
    slots:          Vec<Slot>,
    block_chain:    Arc<RwLock<tmp_blocks>>,
    received_seed_info: Vec<seed_info>,
}

impl EpochProcess {
    pub fn new(mid: Pubkey,eid: u64,seed: u64,b: Arc<RwLock<tmp_blocks>>) -> Self {
        EpochProcess{
            myid:           mid,
            cur_eid:        eid,
            cur_seed:       seed,
            slots:          Vec::new(),
            received_seed_info: Vec::new(),
            block_chain:    b.clone(),
        }
    }
    pub fn start(mut self,state: Arc<RwLock<APOS>>,new_block: TypeNewBlockEvent,
        new_interval: TypeNewTimerIntervalEvent) -> Result<TypeStopEpoch,Error> {
        // setup validators
        match self.assign_validator(state.clone()) {
            Ok(()) => {
                let sid = self.block_chain
                              .read()
                              .expect("acquiring shared_block_chain read lock")
                              .get_sid_from_current_block();
                Ok(self.start_slot_walk_in_epoch(sid,new_block, new_interval, state.clone()))
            },
            Err(e) => Err(e),
        }
    }
    pub fn is_my_produce(&self,sid: i32,state: Arc<RwLock<APOS>>) -> bool {
        if let Some(item) = state.read()
        .expect("acquiring apos read lock")
        .get_staking_holder(sid,self.cur_eid) {
            self.myid.equal(&item.into())
        } else {
            false
        }
    }
    pub fn get_my_pk(&self) -> Option<Pubkey> {
        Some(self.myid.clone())
    }
    pub fn next_epoch(&mut self,sid: i32,state: Arc<RwLock<APOS>>) -> Result<bool,Error> {
        let next_eid = Epoch::get_epoch_from_id(sid,self.cur_eid);
        if next_eid == self.cur_eid + 1 {
            self.cur_eid = next_eid;
            match self.assign_validator(state) {
                Err(e) => Err(e),
                Ok(()) => Ok(true),
            }
        } else {
            Ok(false)
        }
    }
    pub fn assign_validator(&mut self,state: Arc<RwLock<APOS>>) -> Result<(),Error> {
        // if let Some(vals) = state.read()
        // .expect("acquiring apos read lock")
        // .get_staking_holders(self.cur_eid){
        //     self.slots.clear();
        //     let mut validators = vals;
        //     let seed = self.cur_seed;
        //     // fts::assign_valditator_to_slot(&mut validators, seed)?;
        //     // for (i,v) in validators.iter().enumerate() {
        //     //     self.slots.push(
        //     //         slot::new(v.get_sid(),i as u32)
        //     //     );
        //     // }
        //     Ok(())
        // }
        Err(ConsensusErrorKind::NotMatchEpochID.into())
    }
    pub fn slot_handle(&mut self,sid: i32,state: Arc<RwLock<APOS>>) {
        if self.is_my_produce(sid,state) {
           let c_height = self.block_chain
                              .read()
                              .expect("acquiring shared_block_chain read lock")
                              .get_current_Height();
            let c_hash = self.block_chain
                             .read()
                             .expect("acquiring shared_block_chain read lock")
                             .get_hash_by_height(c_height);
            let b = self.block_chain
                        .write()
                        .expect("acquiring shared_block_chain write lock")
                        .make_new_block(c_height,c_hash.unwrap());
            // boradcast the block and insert the block
        }
    }
    pub fn start_slot_walk_in_epoch(mut self,sid: i32,new_block: TypeNewBlockEvent,
        new_interval: TypeNewTimerIntervalEvent,state: Arc<RwLock<APOS>>) -> TypeStopEpoch {
        let (stop_epoch_send, stop_epoch_receiver) = bounded::<()>(1);
        let mut walk_pos :i32 = sid;
        let mut thread_builder = thread::Builder::new();
        // thread_builder = thread_builder.name("slot_walk".to_string());
        let join_handle = thread_builder.spawn(move || loop {
                select! {
                    recv(stop_epoch_receiver) -> _ => {
                        break;
                    }
                    recv(new_block) -> msg => {
                        self.handle_new_block_event(msg,&walk_pos,state.clone());
                        walk_pos = walk_pos + 1;
                    },
                    recv(new_interval) -> _ => {
                        self.handle_new_time_interval_event(&walk_pos,state.clone());
                        walk_pos = walk_pos + 1;
                    },
                }
                // new epoch
                match self.next_epoch(walk_pos,state.clone()) {
                    Err(e) => {
                        println!("start_slot_walk_in_epoch is quit,cause next epoch is err:{:?}",e);
                        return ;
                    },
                    Ok(next) => {
                        if next {
                            walk_pos = 0;
                        }
                    },
                }
            })
            .expect("Start slot_walk failed");
        stop_epoch_send
    }
    fn handle_new_block_event(&mut self, msg: Result<Block, RecvError>, sid: &i32, state: Arc<RwLock<APOS>>) {
        match msg {
            Ok(b) => {
                self.slot_handle(*sid, state.clone());
                self.epoch_step(state,b.height());
            },
            Err(e) => println!("insert_block Error: {:?}", e),
        }
    }
    fn handle_new_time_interval_event(&mut self,sid: &i32,state: Arc<RwLock<APOS>>) {
        self.slot_handle(*sid,state);
    }
    // if it want to be a validator and then make the local secret and broadcast it
    fn commitment_phase(&self, state: Arc<RwLock<APOS>>) -> Result<(),Error> {
        // let seed = match state.read()
        //                       .expect("acquiring apos read lock")
        //                       .get_self_seed() {
        //                         Some(seed) => seed,
        //                         None => {
        //                             state.read()
        //                             .expect("acquiring apos read lock")
        //                             .make_rand_seed()?;
        //                         },
        //             };
        // // update seed
        // seed.update_send_count();
        // if seed.can_send() {
        //     state
        //     .write()
        //     .expect("acquiring apos write lock")
        //     .set_self_seed(Some(seed.clone()));
        //     // broadcast the seed
        //     let s: types::send_seed_info = seed.into();
        //     // send(s.to_bytes())
        // }
        Ok(())
    }
    // broadcast the open info to the validators in the epoch
    fn revel_phase(&self,state: Arc<RwLock<APOS>>) -> Result<(),Error> {
        if let Some(seed) = state.read()
                    .expect("acquiring apos read lock")
                    .get_self_seed() {
            let open = seed.get_Revel_phase_msg();
            // broadcast the open

        }
        Err(ConsensusErrorKind::NotFoundSeedInfo.into())
    }
    // fn receive_shares(&mut self,data: Vec<u8>,state: Arc<RwLock<APOS>>) -> Result<(),Error> {
    //     let obj = types::send_seed_info::from_bytes(data);
    //     let mut find = false;
    //     for elem in self.received_seed_info.iter() {
    //         if elem.same_person(&obj) {
    //             find = true;
    //             break;
    //         }
    //     }
    //     if obj.eid == self.cur_eid && !find {
    //         let mut seed_item = seed_info::from_send_seed_info(&obj);
    //         match state.read()
    //         .expect("acquiring apos read lock")
    //         .recove_the_share(&obj) {
    //             Ok(()) => {
    //                 self.received_seed_info.push(seed_item);
    //                 return Ok(());
    //             },
    //             Err(e) => return Err(e),
    //         }
    //     } else {
    //         return Err(ConsensusErrorKind::NotMatchEpochID.into())
    //     }
    // }
    // recover seed for all holder
    fn recovery_phase(&mut self,state: Arc<RwLock<APOS>>) {

        for seed_item in self.received_seed_info.iter_mut() {
            if !seed_item.is_recover() {
                match state.write()
                .expect("acquiring apos write lock")
                .recover_seed_from_shared_msg(&seed_item) {
                    Ok(data) => {
                        let s = data.as_slice();
                        let a = s[0];
                        let mut b = [0u8;32];
                        b[..].copy_from_slice(&s[1..]);
                        seed_item.set_open_msg(a,&b);
                    },
                    Err(e) => {println!("recover share failed,share:{},error:{:?}",seed_item,e);},
                }
            }
        }
    }

    fn get_seed_info_by_holder(&self, holder: &HolderItem) -> Option<seed_info> {
        // for info in self.received_seed_info {
        //     if info.get_id() == holder.get_my_id() {
        //         return Some(info)
        //     }
        // }
        None
    }

    fn recover_seed_for_next_epoch(&self,state: Arc<RwLock<APOS>>) -> Result<u64,Error> {
        let mut datas: Vec<u8> = Vec::new();

        if let Some(holders) = state.read()
        .expect("acquiring apos read lock")
        .get_staking_holders(self.cur_eid){
            for h in holders.iter() {
                if let Some(info) = self.get_seed_info_by_holder(h) {
                    if info.is_recover() {
                        let msg = info.get_open_msg().to_vec();
                        datas.extend(msg);
                    }
                }
            }
            if datas.len() > 0 {
                let h = Hash::make_hash(datas.as_slice());
                let seed = u64::from_be_bytes(h.0[..8].try_into().unwrap());
                return Ok(seed);
            }
            return Err(ConsensusErrorKind::NotFetchAnyShares.into());
        }
        Err(ConsensusErrorKind::NotMatchEpochID.into())
    }

    pub fn get_current_height(&self) -> u64 {
        return self.block_chain.read().expect("acquiring blockchian read lock").get_current_Height();
    }

    pub fn epoch_step(&mut self, state: Arc<RwLock<APOS>>, height: u64) {
        // get the height event from blockchain
        // 4k,4k,2k for commit phase,revel phase,recovery
        let k = (epoch_length / 10) as u64;
        let m = (height % epoch_length as u64) as u64;
        if m <= 4 * k {
            // commit phase only once send
            self.commitment_phase(state.clone());
        } else if m <= 8*k {
            // revel phase
            self.revel_phase(state.clone());
        } else {
            // recover phase try to recover the seed from shares
            self.recovery_phase(state.clone());
        }
        if 0 as u64 == (height + 1) % epoch_length as u64 {
            if let Ok(seed) = self.recover_seed_for_next_epoch(state.clone()) {
                state.write().expect("acquiring state write lock").set_seed_next_epoch(seed);
            }
        }
        // thread::sleep(Duration::from_millis(2000));
    }
}



#[cfg(test)]
pub mod tests {

    #[test]
    fn make_epoch() {

    }
}