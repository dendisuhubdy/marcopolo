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


use ed25519::{pubkey::Pubkey,privkey::PrivKey,signature::SignatureInfo};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread;
use core::block::{self,Block,BlockProof,VerificationItem};
use crossbeam_channel::{bounded, select, Receiver, RecvError, Sender};
use core::types::{Hash};
use super::{apos::APOS,vrf,types};
use super::vrf;
use super::ConsensusErrorKind;
use errors::{Error,ErrorKind};


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
        let (low,hi) = epoch_info::get_height_from_eid(eid);
        for i in low..hi {
            let b = self.get_best_chain(i);
        }
        0
    }
}
pub struct epoch_info {}
impl epoch_info {
    pub fn get_epoch_from_height(h: u64) -> u64 {
        let eid: u64 = h / epoch_length as u64 + 1;
        eid
    }
    pub fn get_epoch_from_id(sid: i32,cur_eid: u64) -> u64 {
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
pub struct slot {
    timeout:    u32,     // millsecond
    id:         i32,
    vindex:     u32,
}
impl slot {
    pub fn new(sid: i32,index: u32) -> Self {
        slot{
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
    slots:          Vec<slot>,
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
    pub fn make_next_seed() -> u64 {
        // make next seed from blockchain
        0
    }
    pub fn is_my_produce(&self,sid: i32,state: Arc<RwLock<APOS>>) -> bool {
        if let Some(item) = state.read()
        .expect("acquiring apos read lock")
        .get_validator(sid,self.cur_eid) {
            self.myid.equal(&item.into())
        } else {
            false
        }
    }
    pub fn get_my_pk(&self) -> Option<Pubkey> {
        Some(self.myid.clone())
    }
    pub fn next_epoch(&mut self,sid: i32,state: Arc<RwLock<APOS>>) -> Result<bool,Error> {
        let next_eid = epoch_info::get_epoch_from_id(sid,self.cur_eid);
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
        if let Some(vals) = state.read()
        .expect("acquiring apos read lock")
        .get_validators(self.cur_eid){
            self.slots.clear();
            let mut validators = vals;
            let seed = self.cur_seed;
            vrf::assign_valditator_to_slot(&mut validators, seed)?;
            for (i,v) in validators.iter().enumerate() {
                self.slots.push(
                    slot::new(v.get_sid(),i as u32)
                );
            }
            Ok(())
        } 
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
    fn handle_new_block_event(&mut self, msg: Result<Block, RecvError>,sid: &i32,state: Arc<RwLock<APOS>>) {
        match msg {
            Ok(b) => {
                self.slot_handle(*sid,state);
            },
            Err(e) => println!("insert_block Error: {:?}", e),
        }
    }
    fn handle_new_time_interval_event(&mut self,sid: &i32,state: Arc<RwLock<APOS>>) {
        self.slot_handle(*sid,state);
    }
    // if it want to be a validator and then make the local secret and broadcast it
    fn commitment_phase(&self,state: Arc<RwLock<APOS>>) -> Result<(),Error> {
        let seed = state.read()
                    .expect("acquiring apos read lock")
                    .make_rand_seed()?;
        // update seed 
        state
        .write()
        .expect("acquiring apos write lock")
        .set_self_seed(Some(seed.clone()));
        // broadcast the seed
        let s: types::send_seed_info = seed.into();
        // send(s.to_bytes())
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
    fn receive_shares(&mut self,data: Vec<u8>,state: Arc<RwLock<APOS>>) -> Result<(),Error> {
        let obj = types::send_seed_info::from_bytes(data);
        state.read()
            .expect("acquiring apos read lock")
            .recove_the_share(&mut obj) {
            Ok(info) => {
                // remove repeat
                self.received_seed_info.push(info),
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
    fn recovery_phase(&self,state: Arc<RwLock<APOS>>) {
        
    }
}



#[cfg(test)]
pub mod tests {
    
    #[test]
    fn make_epoch() {

    }
}