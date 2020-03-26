use std::collections::HashMap;
use map_core::types::Hash;
use map_core::block::Block;
use map_core::transaction::Transaction;

#[derive(Clone)]
pub struct TxPoolManager {
    txs: HashMap<Hash, Transaction>,
}

impl TxPoolManager {
    pub fn submit_txs(&mut self, tx: Transaction)  {
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

    pub fn start() -> TxPoolManager {
        TxPoolManager {
            txs: HashMap::new(),
        }
    }
}
