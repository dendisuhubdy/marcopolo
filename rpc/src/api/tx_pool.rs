use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use map_core::transaction::Transaction;

/// TxPool rpc interface.
#[rpc(server)]
pub trait TxPool {
    /// Returns protocol version.
    #[rpc(name = "send_transaction")]
    fn send_transaction(&self) -> Result<String>;
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
    fn send_transaction(&self) -> Result<String> {
        Ok(format!("{}", self.txs.len()))
    }
}