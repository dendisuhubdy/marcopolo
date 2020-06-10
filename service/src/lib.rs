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

extern crate chain;
extern crate consensus;
// You should have received a copy of the GNU General Public License
// along with MarcoPolo Protocol.  If not, see <http://www.gnu.org/licenses/>.
extern crate core;
extern crate errors;
extern crate executor;
#[macro_use]
extern crate log;
extern crate network;
extern crate rpc;

use std::{sync::mpsc, thread, thread::JoinHandle};
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, SystemTime};

use chain::blockchain::BlockChain;
use chain::tx_pool::TxPoolManager;
use ed25519::pubkey::Pubkey;
use ed25519::privkey::PrivKey;
use consensus::{ConsensusErrorKind, poa::POA};
use core::balance::Balance;
use core::block::{self, Block, Header};
use core::genesis::{ed_genesis_priv_key, ed_genesis_pub_key};
use core::types::Hash;
use core::runtime::Interpreter;
use errors::Error;
use executor::Executor;
use generator::epoch::EpochProcess;
use generator::apos::APOS;
use network::{manager as network_executor, Multiaddr, NetworkConfig};
use rpc::http_server;
use futures::{Future};

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub log: String,
    pub data_dir: PathBuf,
    pub rpc_addr: String,
    pub rpc_port: u16,
    pub key: String,
    pub poa_privkey: String,
    /// List of p2p nodes to initially connect to.
    pub dial_addrs: Vec<Multiaddr>,
    pub p2p_port: u16,
    pub seal_block: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            log: "info".into(),
            data_dir: PathBuf::from("."),
            rpc_addr: "127.0.0.1".into(),
            rpc_port: 9545,
            key: "".into(),
            poa_privkey: "".into(),
            dial_addrs: vec![],
            p2p_port: 40313,
            seal_block:false,
        }
    }
}

//#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Service {
    pub block_chain: Arc<RwLock<BlockChain>>,
    pub tx_pool : Arc<RwLock<TxPoolManager>>,
    pub cfg: NodeConfig,
}

impl Service {
    pub fn new_service(cfg: NodeConfig) -> Self {
        let chain = Arc::new(RwLock::new(BlockChain::new(cfg.data_dir.clone(),cfg.poa_privkey.clone())));
        Service {
            block_chain: chain.clone(),
            tx_pool: Arc::new(RwLock::new(TxPoolManager::start(chain.clone()))),
            cfg:   cfg.clone(),
        }
    }
    fn get_POA(&self) -> POA {
        let key = self.cfg.poa_privkey.clone();
        POA::new_from_string(key)
    }
    pub fn start(mut self,cfg: NodeConfig) -> (mpsc::Sender<i32>,JoinHandle<()>) {
        self.get_write_blockchain().load();
        let network_block_chain = self.block_chain.clone();
        let thread_cfg = cfg.clone();

        let mut config = NetworkConfig::new();
        config.update_network_cfg(cfg.data_dir, cfg.dial_addrs, cfg.p2p_port).unwrap();
        let mut network = network_executor::NetworkExecutor::new(config.clone(), network_block_chain).expect("Network start error");

        let rpc = http_server::start_http(http_server::RpcConfig {
            rpc_addr: cfg.rpc_addr,
            rpc_port: cfg.rpc_port,
            key: cfg.key.clone(),
        }, self.block_chain.clone(), self.tx_pool.clone());

        let (tx,rx): (mpsc::Sender<i32>,mpsc::Receiver<i32>) = mpsc::channel();
        let shared_block_chain = self.block_chain.clone();

        let node_key = match PrivKey::from_hex(&cfg.key.clone()) {
            Ok(k) => k.to_pubkey().unwrap(),
            _ => Pubkey::from_hex("0xf3a87c2ea52bbc7cd764ddd7f947d93ce20d094872185049761ffb2652c09307"),
        };

        let slot_tick = EpochProcess::new(
            node_key,
            0,
            0,
            shared_block_chain.clone(),
        );
        let stake = APOS::new(shared_block_chain.clone());
        slot_tick.start(Arc::new(RwLock::new(stake)));

        let builder = thread::spawn(move || {
            loop {
                if !thread_cfg.seal_block {
                    let res2 = self.generate_block();
                    match res2 {
                        Ok(b) => {
                            if let Err(e) = shared_block_chain
                                .write()
                                .expect("acquiring shared_block_chain write lock")
                                .insert_block(b.clone()) {
                                error!("insert_block Error: {:?}", e);
                            } else {
                                network.gossip(b);
                            }
                        },
                        Err(e) => error!("generate_block,Error: {:?}", e),
                    };
                    thread::sleep(Duration::from_millis(POA::get_interval()));
                }

                if rx.try_recv().is_ok() {
                    if !network.exit_signal.is_closed() {
                        network.exit_signal.send(1).expect("network exit error");
                    }
                    network.runtime
                        .shutdown_on_idle()
                        .wait()
                        .map_err(|e| format!("Tokio runtime shutdown returned an error: {:?}", e)).unwrap();
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
        let finalize = self.get_POA();
        let chain = self.block_chain.read().unwrap();
        let statedb = chain.state_at(cur_block.state_root());

        let h = Executor::exc_txs_in_block(&b, &mut Balance::new(Interpreter::new(statedb)), &POA::get_default_miner())?;
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
    use std::fmt;

    use super::*;

    #[test]
    fn test_service() {
        println!("begin service,for 60 seconds");
        let mut config = NodeConfig::default();
        let service = Service::new_service(config.clone());
        let (tx,th_handle) = service.start(config.clone());
        thread::sleep(Duration::from_millis(60*1000));
        thread::spawn(move || {
            tx.send(1).unwrap();
        });
        th_handle.join();
        println!("end service");
    }
}
