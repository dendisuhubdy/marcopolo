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

use std::convert::TryInto;
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime};

use crate::types::{seed_info, HolderItem};
use crate::{apos::APOS, types};
use chain::blockchain::BlockChain;
use crossbeam_channel::{bounded, unbounded, select, tick, Receiver, RecvError, Sender};
use futures::future::Future;
// use tokio::sync::mpsc::{Receiver, Sender};
use ed25519::{privkey::PrivKey, pubkey::Pubkey, signature::SignatureInfo};
use errors::{Error, ErrorKind};
use map_consensus::ConsensusErrorKind;
use map_network::manager::NetworkExecutor;
use map_core::block::{self, Block, BlockProof, VerificationItem};
use map_core::types::Hash;
// use super::fts;

/// Slots per epoch constant
pub const EPOCH_LENGTH: u64 = 64;
pub const SLOT_DURATION: u64 = 6;

type TypeNewBlockEvent = Receiver<Block>;
type TypeNewTimerIntervalEvent = Receiver<Instant>;
type TypeTickEvent = Receiver<Instant>;
pub type TypeStopEpoch = Sender<()>;

// Chain bulder to make proposer block
#[derive(Clone)]
pub struct Builder {
    chain: Arc<RwLock<BlockChain>>,
}

impl Builder {
    pub fn new(chain: Arc<RwLock<BlockChain>>) -> Self {
        Builder { chain: chain }
    }
    // Proposal new block from certain slot
    pub fn make_new_block(&self, height: u64, parent: Hash) -> Block {
        let pre = self.chain.read().unwrap().get_block(parent).unwrap();
        let mut block = Block::default();
        block.header.parent_hash = parent;
        block.header.height = height + 1;
        block.header.tx_root = Hash::default();
        block.header.state_root = pre.state_root();
        block.header.sign_root = Hash::default();
        block.header.time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        block
    }

    pub fn get_current_height(&self) -> u64 {
        self.chain.read().unwrap().current_block().height()
    }

    pub fn get_head_block(&self) -> Block {
        self.chain.read().unwrap().current_block()
    }

    pub fn get_block_by_height(&self, height: u64) -> Option<Block> {
        self.chain.read().unwrap().get_block_by_number(height)
    }

    pub fn get_sid_from_current_block(&self) -> u64 {
        self.chain.read().unwrap().current_block().height() + 1
    }

    pub fn get_best_chain(&self, height: u64) -> Option<Block> {
        Some(Block::default())
    }

    pub fn get_blockchain(&self) ->  Arc<RwLock<BlockChain>> {
        // self.chain.write().unwrap().insert_block_ref(block);
        self.chain.clone()
    }

    // pub fn insert_block(&self, block: &Block) {
    //     self.chain.write().unwrap().insert_block_ref(block);
    // }

    pub fn make_seed_in_epoch(&self, eid: u64) -> u64 {
        let (low, hi) = Epoch::get_height_from_eid(eid);
        for i in low..hi {
            let b = self.get_best_chain(i);
        }
        0
    }
}

pub struct Epoch(u64);

impl Epoch {
    pub fn epoch_from_height(h: u64) -> u64 {
        let eid: u64 = h / EPOCH_LENGTH;
        eid
    }

    pub fn epoch_from_id(sid: u64) -> u64 {
        // We may skip block from slot
        let eid: u64 = sid / EPOCH_LENGTH;
        eid
    }

    pub fn get_height_from_eid(eid: u64) -> (u64, u64) {
        if eid as i64 <= 0 {
            return (0, 0);
        }
        let low: u64 = (eid - 1) * EPOCH_LENGTH as u64;
        let hi: u64 = eid * EPOCH_LENGTH as u64 - 1;
        (low, hi)
    }
}

#[derive(Debug, Clone)]
pub struct Slot {
    timeout: u32, // millsecond
    id: i32,
    vindex: u32,
}

impl Slot {
    pub fn new(sid: i32, index: u32) -> Self {
        Slot {
            timeout: 5000,
            id: sid,
            vindex: index,
        }
    }
}

pub struct EpochProcess {
    exit_event: Receiver<i32>,
    myid: Pubkey,
    cur_eid: u64,
    cur_seed: u64,
    slots: Vec<Slot>,
    block_chain: Builder,
    received_seed_info: Vec<seed_info>,
    network: NetworkExecutor,
}

impl EpochProcess {
    pub fn new(mid: Pubkey, eid: u64, seed: u64, chain: Arc<RwLock<BlockChain>>, p2p: NetworkExecutor, exit: Receiver<i32>) -> Self {
        EpochProcess {
            myid: mid,
            cur_eid: eid,
            cur_seed: seed,
            slots: Vec::new(),
            received_seed_info: Vec::new(),
            block_chain: Builder::new(chain.clone()),
            exit_event: exit,
            network: p2p,
        }
    }

    pub fn start(
        mut self,
        state: Arc<RwLock<APOS>>,
    ) -> JoinHandle<()> {
        // let new_interval = tick(Duration::new(6, 0));
        // setup validators
        // match self.assign_validator(state.clone()) {
        //     Ok(()) => {
        //         let sid = self.block_chain.get_sid_from_current_block();
        //         Ok(self.start_slot_walk_in_epoch(sid, new_block, new_interval, state.clone()))
        //     }
        //     Err(e) => Err(e),
        // }

        let (_, new_block) = unbounded();

        // Get start slot on node lanuch
        let sid = self.block_chain.get_sid_from_current_block();
        self.start_slot_walk_in_epoch(sid, new_block, state.clone())
    }

    pub fn is_proposer(&self, sid: u64, state: Arc<RwLock<APOS>>) -> bool {
        if let Some(item) = state
            .read()
            .expect("acquiring apos read lock")
            .get_staking_holder(sid, self.cur_eid)
        {
            self.myid.equal(&item.into())
        } else {
            false
        }
    }

    pub fn get_my_pk(&self) -> Option<Pubkey> {
        Some(self.myid.clone())
    }

    pub fn next_epoch(&mut self, sid: u64, state: Arc<RwLock<APOS>>) -> Result<bool, Error> {
        let next_eid = Epoch::epoch_from_id(sid);
        if next_eid == self.cur_eid + 1 {
            self.cur_eid = next_eid;
            // match self.assign_validator(state) {
            //     Err(e) => Err(e),
            //     Ok(()) => Ok(true),
            // }
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn assign_validator(&mut self, state: Arc<RwLock<APOS>>) -> Result<(), Error> {
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
        // Err(ConsensusErrorKind::NotMatchEpochID.into())
        let pos = state.read().unwrap();
        let committee = pos.get_epoch_info(self.cur_eid).unwrap();

        Ok(())
    }

    pub fn new_slot_handle(&mut self, sid: u64, state: Arc<RwLock<APOS>>) {
        info!("new slot id={}", sid);
        if self.is_proposer(sid, state) {
            let current = self.block_chain.get_head_block();
            let b = self
                .block_chain
                .make_new_block(current.height(), current.hash());
            info!("make new block hash={} num={}", b.hash(), b.height());

            let block_chain = self.block_chain.get_blockchain();
            let mut chain = block_chain.write().unwrap();
            if let Err(e) = chain.insert_block(b.clone()) {
                error!("insert_block Error: {:?}", e);
            }
            // boradcast and import the block
            self.network.gossip(b);
        }
    }

    pub fn slot_handle(&mut self, sid: u64, state: Arc<RwLock<APOS>>) {
        if self.is_proposer(sid, state) {
            let current = self.block_chain.get_head_block();
            let b = self
                .block_chain
                .make_new_block(current.height(), current.hash());
            info!("make new block hash={}", b.hash());
            // boradcast and import the block
        }
    }

    pub fn start_slot_walk_in_epoch(
        mut self,
        sid: u64,
        new_block: TypeNewBlockEvent,
        state: Arc<RwLock<APOS>>,
    ) -> JoinHandle<()> {
        let (stop_epoch_send, stop_epoch_receiver) = bounded::<()>(1);
        let mut walk_pos: u64 = sid;
        let thread_builder = thread::Builder::new();
        // let start_slot = sid;
        let new_interval = tick(Duration::new(SLOT_DURATION, 0));
        // let start = Instant::now();
        // let now = Instant::now();
        // let elapse = now.duration_since(start);

        let join_handle = thread_builder
            .spawn(move || loop {

                select! {
                    // recv(stop_epoch_receiver) -> _ => {
                    //     // end of slot
                    //     // break;
                    //     warn!("stop receiver");
                    // },
                    recv(new_interval) -> _ => {
                        self.handle_new_time_interval_event(walk_pos, state.clone());
                        walk_pos = walk_pos + 1;
                    },
                    recv(self.exit_event) -> _ => {
                        warn!("slot tick task exit");
                        if !self.network.exit_signal.is_closed() {
                            self.network.exit_signal.send(1).expect("network exit error");
                        }
                        self.network.runtime
                            .shutdown_on_idle()
                            .wait()
                            .map_err(|e| format!("Tokio runtime shutdown returned an error: {:?}", e)).unwrap();
                        break;
                    },
                    // recv(new_block) -> msg => {
                    //     self.handle_new_block_event(msg, &walk_pos, state.clone());
                    //     walk_pos = walk_pos + 1;
                    // },
                }
                // new epoch
                // match self.next_epoch(walk_pos + 1, state.clone()) {
                //     Err(e) => {
                //         println!(
                //             "start_slot_walk_in_epoch is quit,cause next epoch is err:{:?}",
                //             e
                //         );
                //         return;
                //     },
                //     _ => (),
                // }

                // No skipping empty slot right now
                // walk_pos = self.block_chain.get_sid_from_current_block();
            })
            .expect("Start slot_walk failed");
        join_handle
    }

    fn handle_new_block_event(
        &mut self,
        msg: Result<Block, RecvError>,
        sid: &u64,
        state: Arc<RwLock<APOS>>,
    ) {
        match msg {
            Ok(b) => {
                self.slot_handle(*sid, state.clone());
                self.epoch_step(state, b.height());
            }
            Err(e) => println!("insert_block Error: {:?}", e),
        }
    }
    fn handle_new_time_interval_event(&mut self, sid: u64, state: Arc<RwLock<APOS>>) {
        self.new_slot_handle(sid, state);
    }
    // if it want to be a validator and then make the local secret and broadcast it
    fn commitment_phase(&self, state: Arc<RwLock<APOS>>) -> Result<(), Error> {
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
    fn revel_phase(&self, state: Arc<RwLock<APOS>>) -> Result<(), Error> {
        // if let Some(seed) = state.read()
        //             .expect("acquiring apos read lock")
        //             .get_self_seed() {
        //     let open = seed.get_Revel_phase_msg();
        //     // broadcast the open

        // }
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
    fn recovery_phase(&mut self, state: Arc<RwLock<APOS>>) {
        for seed_item in self.received_seed_info.iter_mut() {
            if !seed_item.is_recover() {
                match state
                    .write()
                    .expect("acquiring apos write lock")
                    .recover_seed_from_shared_msg(&seed_item)
                {
                    Ok(data) => {
                        let s = data.as_slice();
                        let a = s[0];
                        let mut b = [0u8; 32];
                        b[..].copy_from_slice(&s[1..]);
                        seed_item.set_open_msg(a, &b);
                    }
                    Err(e) => {
                        println!("recover share failed,share:{},error:{:?}", seed_item, e);
                    }
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

    fn recover_seed_for_next_epoch(&self, state: Arc<RwLock<APOS>>) -> Result<u64, Error> {
        let mut datas: Vec<u8> = Vec::new();

        if let Some(holders) = state
            .read()
            .expect("acquiring apos read lock")
            .get_staking_holders(self.cur_eid)
        {
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

    pub fn epoch_step(&mut self, state: Arc<RwLock<APOS>>, height: u64) {
        // get the height event from blockchain
        // 4k,4k,2k for commit phase,revel phase,recovery
        let k = (EPOCH_LENGTH / 10) as u64;
        let m = (height % EPOCH_LENGTH as u64) as u64;
        if m <= 4 * k {
            // commit phase only once send
            self.commitment_phase(state.clone());
        // } else if m <= 8*k {
        //     // revel phase
        //     self.revel_phase(state.clone());
        } else {
            // recover phase try to recover the seed from shares
            self.recovery_phase(state.clone());
        }
        if 0 as u64 == (height + 1) % EPOCH_LENGTH as u64 {
            if let Ok(seed) = self.recover_seed_for_next_epoch(state.clone()) {
                state
                    .write()
                    .expect("acquiring state write lock")
                    .set_seed_next_epoch(seed);
            }
        }
        // thread::sleep(Duration::from_millis(2000));
    }
}

#[cfg(test)]
pub mod tests {
    use std::thread;
    use std::time::{Duration, Instant};
    use crossbeam_channel::tick;

    #[test]
    fn slot_tick() {
        let start = Instant::now();
        let ticker = tick(Duration::from_millis(1000));

        for _ in 0..2 {
            ticker.recv().unwrap();
            println!("elapsed: {:?}", start.elapsed());
        }
        thread::sleep(Duration::from_millis(1500));

        for _ in 0..2 {
            ticker.recv().unwrap();
            println!("delayed elapsed: {:?}", start.elapsed());
        }
    }
}
