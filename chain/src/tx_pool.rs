use std::collections::HashMap;

use futures::{Async, Future, Sink, Stream};
use futures::sync::mpsc;

use map_core::transaction::Transaction;

#[derive(Clone)]
pub struct TxPoolManager {
    sender: mpsc::Sender<Transaction>,
    // receiver: mpsc::Receiver<Transaction>,
    txs: Vec<Transaction>,
}

impl TxPoolManager {
    pub fn submit_txs(&self, tx: Transaction) {
        let mut sender = self.sender.clone();
        sender.send(tx).wait().expect("unable to send");
    }

    pub fn start() -> TxPoolManager {
        let (sender, receiver) = mpsc::channel::<Transaction>(512);
        TxPoolManager {
            sender,
            txs: Vec::new(),
        }
    }
}
