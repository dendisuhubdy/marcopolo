use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use map_core::balance::Balance;
use map_core::block::Block;
use map_core::transaction::Transaction;
use map_core::types::{Address, Hash};
use map_core::runtime::Interpreter;
use crate::blockchain::BlockChain;

#[derive(Clone)]
pub struct TxPoolManager {
    txs: HashMap<Hash, Transaction>,
    blockchain: Arc<RwLock<BlockChain>>,
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

    pub fn start(chain: Arc<RwLock<BlockChain>>) -> TxPoolManager {
        TxPoolManager {
            txs: HashMap::new(),
            blockchain: chain,
        }
    }

    fn validate_tx(&self, tx: &Transaction) -> Result<(), String> {
        let chain = self.blockchain.read().unwrap();
        let state = chain.state_at(chain.current_block().state_root());
        let runtime = Balance::new(Interpreter::new(state));
        let account = runtime.get_account(tx.sender);

        if account.get_balance() < tx.get_value() {
            return Err(format!("not sufficient funds {}, tx value {}", account.get_balance(), tx.get_value()));
        }

        if account.get_nonce() + 1 != tx.get_nonce() {
            return Err(format!("invalid nonce {}, tx value {}", account.get_nonce(), tx.get_nonce()));
        }
        Ok(())
    }

    pub fn get_nonce(&self, addr: &Address) -> u64 {
        let chain = self.blockchain.read().unwrap();
        let state = chain.state_at(chain.current_block().state_root());
        let runtime = Balance::new(Interpreter::new(state));
        let account = runtime.get_account(addr.clone());

        account.get_nonce()
    }
}