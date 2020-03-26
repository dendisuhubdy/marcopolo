use std::collections::HashMap;

use futures::{Async, Future, Sink, Stream};
use futures::sync::mpsc;

use map_core::transaction::Transaction;

#[derive(Clone)]
pub struct TxPoolManager {
    txs: Vec<Transaction>,
}

impl TxPoolManager {
    pub fn submit_txs(&mut self, tx: Transaction) {
        self.txs.push(tx);
    }

    pub fn get_txs(&self) -> &Vec<Transaction> {
        &self.txs
    }

    pub fn start() -> TxPoolManager {
        TxPoolManager {
            txs: Vec::new(),
        }
    }
}
