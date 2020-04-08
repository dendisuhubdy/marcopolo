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
extern crate consensus;
extern crate chain;
extern crate rpc;
extern crate network;
extern crate executor;
#[macro_use]
extern crate log;
extern crate errors;

use core::block::{self,Block,Header};
use core::types::Hash;
use core::balance::Balance;
use core::genesis::{ed_genesis_priv_key,ed_genesis_pub_key};
use consensus::{poa::POA,ConsensusErrorKind};
use chain::blockchain::{BlockChain};
use chain::tx_pool::TxPoolManager;
use executor::Executor;
use rpc::http_server;
use network::handler;
use std::{thread,thread::JoinHandle,sync::mpsc};
use std::time::{Duration, SystemTime};
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use errors::Error;

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub log: String,
    pub data_dir: PathBuf,
    pub rpc_addr: String,
    pub rpc_port: u16,
    pub key:      String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            log: "info".into(),
            data_dir: PathBuf::from("."),
            rpc_addr:"127.0.0.1".into(),
            rpc_port:9545,
            key:    "".into(),
        }
    }
}

// pub mod Service;

//#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Service {
    pub block_chain: Arc<RwLock<BlockChain>>,
    pub state: Arc<RwLock<Balance>>,
    pub tx_pool : Arc<RwLock<TxPoolManager>>,
}

impl Service {
    pub fn new_service(cfg: NodeConfig) -> Self {
        let state = Arc::new(RwLock::new(Balance::new(cfg.data_dir.clone())));

        Service {
            block_chain: Arc::new(RwLock::new(BlockChain::new(cfg.data_dir.clone()))),
            state: state.clone(),
            tx_pool: Arc::new(RwLock::new(TxPoolManager::start(state.clone()))),
        }
    }
    pub fn start(mut self,cfg: NodeConfig) -> (mpsc::Sender<i32>,JoinHandle<()>) {
        {
            let mut statedb = self.state.write().unwrap();
            self.get_write_blockchain().load(&mut statedb);
        }
        handler::start_network("40313");
        let rpc = http_server::start_http(http_server::RpcConfig{
            rpc_addr:cfg.rpc_addr,
            rpc_port:cfg.rpc_port,
            key:cfg.key,
        },self.block_chain.clone(),self.tx_pool.clone());

        let (tx,rx): (mpsc::Sender<i32>,mpsc::Receiver<i32>) = mpsc::channel();
        let shared_block_chain = self.block_chain.clone();

        let builder = thread::spawn(move || {
            loop {
                let res2 = self.generate_block();
                match res2 {
                    Ok(b) => {
                        if let Err(e) = shared_block_chain
                            .write()
                            .expect("acquiring shared_block_chain write lock")
                            .insert_block(b) {
                                error!("insert_block Error: {:?}", e);
                            }
                    },
                    Err(e) => error!("generate_block,Error: {:?}", e),
                };
                thread::sleep(Duration::from_millis(POA::get_interval()));
                if rx.try_recv().is_ok() {
                    rpc.close();
                    break;
                }
            }
        });
        (tx,builder)
    }
    pub fn new_empty_block() -> Block {
        Block::default()
    }
    pub fn generate_block(&mut self) -> Result<Block,Error> {
        let cur_block = self.get_write_blockchain().current_block();
        let tx_pool = self.tx_pool.clone();

        let txs =
            tx_pool.read().expect("acquiring tx_pool read lock").get_txs();

        let txs_root = block::get_hash_from_txs(&txs);
        let header: Header = Header{
            height: cur_block.height() + 1,
            parent_hash: cur_block.get_hash().clone(),
            tx_root:    txs_root,
            state_root: Hash([0;32]),
            sign_root:  Hash([0;32]),
			time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
        };
        info!("seal block, height={}, parent={}, tx={}", header.height, header.parent_hash, txs.len());
        let b = Block::new(header,txs,Vec::new(),Vec::new());
        let finalize = POA{};
        let mut statedb = self.state.write().unwrap();
        let h = Executor::exc_txs_in_block(&b, &mut statedb, &POA::get_default_miner())?;
        tx_pool.write().expect("acquiring tx_pool write lock").notify_block(&b);
        finalize.finalize_block(b,h)
    }
    pub fn get_current_block(&mut self) -> Block {
        self.get_write_blockchain().current_block()
    }
    pub fn get_current_height(&mut self) -> u64 {
        self.get_write_blockchain().current_block().height()
    }
    pub fn get_block_by_height(&self,height: u64) -> Option<Block> {
        self.get_readblockchain().get_block_by_number(height)
    }

    fn get_readblockchain(&self) -> RwLockReadGuard<BlockChain> {
        self.block_chain.read().expect("acquiring block_chain read lock")
    }

    fn get_write_blockchain(&self) -> RwLockWriteGuard<BlockChain> {
        self.block_chain.write().expect("acquiring block_chain write lock")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[test]
    fn test_service() {
        println!("begin service,for 60 seconds");
        let mut config = NodeConfig::default();
        let service = Service::new_service(config);
        let (tx,th_handle) = service.start(config.clone());
        thread::sleep(Duration::from_millis(60*1000));
        thread::spawn(move || {
            tx.send(1).unwrap();
        });
        th_handle.join();
        println!("end service");
    }
}
