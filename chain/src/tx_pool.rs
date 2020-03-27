use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use map_core::balance::Balance;
use map_core::block::Block;
use map_core::transaction::Transaction;
use map_core::types::{Address, Hash};

#[derive(Clone)]
pub struct TxPoolManager {
    txs: HashMap<Hash, Transaction>,
    state: Arc<RwLock<Balance>>,
}

impl TxPoolManager {
    pub fn submit_txs(&mut self, tx: Transaction) {
        match self.validate_tx(&tx) {
            Ok(_) => self.txs.insert(tx.hash(), tx),
            Err(e) => {
                return println!("submit_txs {}", e.as_str());
            }
        };
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

    fn validate_tx(&self, tx: &Transaction) -> Result<(), String> {
        let account = self.state.read().expect("state lock").get_account(tx.sender);
        println!("balance {}, nonce {} ", account.get_balance(), account.get_nonce());

        if account.get_balance() < tx.get_value() {
            return Err(format!("not sufficient funds {}, tx value {}", account.get_balance(), tx.get_value()));
        }

        if account.get_nonce() + 1 != tx.get_nonce() {
            return Err(format!("invalid nonce {}, tx value {}", account.get_nonce(), tx.get_nonce()));
        }
        Ok(())
    }

    pub fn get_nonce(&self, addr: &Address) -> u64 {
        let account = self.state.read().expect("state lock").get_account(addr.clone());
        account.get_nonce()
    }
}