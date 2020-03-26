use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use map_core::balance::Balance;
use map_core::block::Block;
use map_core::transaction::Transaction;
use map_core::types::Hash;

#[derive(Clone)]
pub struct TxPoolManager {
    txs: HashMap<Hash, Transaction>,
    state: Arc<RwLock<Balance>>,
}

impl TxPoolManager {
    pub fn submit_txs(&mut self, tx: Transaction) {
        self.validate_tx(&tx);
        self.txs.insert(tx.hash(), tx);
    }

    pub fn get_txs(&self) -> Vec<Transaction> {
        self.txs.values().cloned().collect()
    }

    pub fn notify_block(&mut self, b: &Block) {
        for tx in &b.txs {
            self.txs.remove(&tx.hash());
        }
    }

    pub fn start(state: Arc<RwLock<Balance>>) -> TxPoolManager {
        TxPoolManager {
            txs: HashMap::new(),
            state,
        }
    }

    fn validate_tx(&self, tx: &Transaction){
        println!("balance {} ",self.state.read().expect("state lock").balance(tx.sender))
    }
}