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

const epoch_length: i32 = 100;
struct tmp_blocks {}
impl tmp_blocks {

}
#[derive(Debug, Clone)]
pub struct slot {
    timeout:    u32,     // millsecond
    id:         u32,
    vindex:     u32,
}
impl slot {
    pub fn new(id: u32,vindex: u32) -> Self {
        slot{
            timeout:    5000,
            id:         id,
            vindex:     vindex,    
        }
    }
}
pub struct EpochProcess {
    myid:       Pubkey,
    cur_eid:    u64,
    cur_seed:       u64,
    slots:      Vec<slot>,
}

impl EpochProcess {
    pub fn new(mid: Pubkey,eid: u64,seed: u64) -> Self {
        EpochProcess{
            myid:   mid,
            cur_eid: eid,
            cur_seed:    seed,
            slots: Vec::new(),
        }
    }
    pub fn vrf(seed: u64,eid: u64,sid: u32,validators: &Vec<ValidatorItem>) -> i32 {
        0
    }
    pub fn is_my_produce(&self) -> bool {
        true
    }
    pub fn get_my_pk(&self) -> Option<Pubkey> {
        None   
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
    pub fn slot_handle(&mut self,id: u32) {
        
    }
}