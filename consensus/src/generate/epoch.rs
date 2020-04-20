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


const epoch_length: i32 = 100;
type TypeNewBlockEvent = Receiver<Block>; 
type TypeNewTimerIntervalEvent = Receiver<()>;
pub type TypeStopEpoch = Sender<()>;


struct tmp_blocks {}
impl tmp_blocks {
    pub fn make_new_block(&self,height: u64,h: Hash) -> Option<Block> {
        Block::default()
    }
    pub fn get_current_Height(&self) -> u64 {
        0
    } 
    pub fn get_hash_by_height(&self,height: u64) -> Option<Hash> {
        Hash([0u8;32])
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
    block_chain:    Arc<RwLock<tmp_blocks>>
}

impl EpochProcess {
    pub fn new(mid: Pubkey,eid: u64,seed: u64,b: &Arc<RwLock<tmp_blocks>>) -> Self {
        EpochProcess{
            myid:           mid,
            cur_eid:        eid,
            cur_seed:       seed,
            slots:          Vec::new(),
            block_chain:    b,
        }
    }
    pub fn vrf(seed: u64,eid: u64,sid: u32,validators: &Vec<ValidatorItem>) -> i32 {
        0
    }
    pub fn is_my_produce(&self,sid: i32,state: &APOS) -> bool {
        if let Some(item) = state.get_validator(sid,self.cur_eid) {
            self.myid.equal(&item.into())
        } else {
            false
        }
    }
    pub fn get_my_pk(&self) -> Option<Pubkey> {
        Some(self.myid)
    }
    pub fn assign_validator(&mut self,state: &APOS) -> Result<(),Error> {
        if let Some(&vals) = state.get_validators(self.cur_eid){
            self.slots.clear();
            for i in 0..vals.len() {
                self.slots.push(
                    slot::new(i,EpochProcess::vrf(self.cur_seed,self.cur_eid,i,vals))
                );
            }
            Ok(())
        } else {
            Err(ConsensusErrorKind::NotMatchEpochID.into())
        } 
    }
    pub fn slot_handle(&mut self,sid: i32,state: &APOS) {
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
                        .make_new_block(c_height,c_hash);
            // boradcast the block and insert the block
        }
    }
    pub fn start_slot_walk_in_epoch(mut self,sid: i32,new_block: &TypeNewBlockEvent,
        new_interval: &TypeNewTimerIntervalEvent,state: &APOS) -> TypeStopEpoch {
        let (stop_epoch_send, stop_epoch_receiver) = bounded::<()>(1);
        let mut walk_pos :i32 = sid;
        let mut thread_builder = thread::Builder::new();
        thread_builder = thread_builder.name("slot_walk".to_string());
        let join_handle = thread_builder
            .spawn(move || loop {
                select! {
                    recv(stop_epoch_receiver) -> _ => {
                        break;
                    }
                    recv(new_block) -> msg => {
                        self.handle_new_block_event(msg,walk_pos,state);
                        walk_pos = walk_pos + 1;
                    },
                    recv(new_interval) -> _ => {
                        self.handle_new_time_interval_event(walk_pos,state);
                        walk_pos = walk_pos + 1;
                    },
                }
            })
            .expect("Start slot_walk failed");  
        stop_epoch_send
    }
    fn handle_new_block_event(&mut self, msg: Result<Block, RecvError>,sid: &i32,state: &APOS) {
        match msg {
            Ok(b) => {
                self.slot_handle(sid,state);
            },
            Err(e) => println!("insert_block Error: {:?}", e),
        }
    }
    fn handle_new_time_interval_event(&mut self,sid: &i32,state: &APOS) {
        self.slot_handle(sid,state);
    }
}

