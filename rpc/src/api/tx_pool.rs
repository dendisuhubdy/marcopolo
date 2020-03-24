use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use map_core::transaction::Transaction;
use bytes::Bytes;
use map_core::types::Address;

/// TxPool rpc interface.
#[rpc(server)]
pub trait TxPool {
    /// Returns protocol version.
    #[rpc(name = "map_sendTransaction")]
    fn send_transaction(&self, from: String, to: String, value: u64) -> Result<String>;
}

/// TxPool rpc implementation.
pub struct TxPoolClient {
    pub txs: Vec<Transaction>,
}

impl TxPoolClient {
    /// Creates new NetClient.
    pub fn new() -> Self {
        TxPoolClient {
            txs: Vec::new(),
        }
    }
}

impl TxPool for TxPoolClient {
    fn send_transaction(&self, from: String, to: String, value: u64) -> Result<String> {
        let f = Address::default();
        let t = Address::default();
        let b = Bytes::new();
        let tx = Transaction::new(f,t,1,1000,1000,value,b);
//        self.txs.push(tx);
        Ok(format!("{}", tx.hash()))
    }
}