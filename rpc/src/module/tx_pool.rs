use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

/// TxPool rpc interface.
#[rpc(server)]
pub trait TxPool {
    /// Returns protocol version.
    #[rpc(name = "send_transaction")]
    fn send_transaction(&self) -> Result<String>;
}

/// TxPool rpc implementation.
pub(crate) struct TxPoolClient {
    tx_count: u64,
}

impl TxPoolClient {
    /// Creates new NetClient.
    pub fn new() -> Self {
        TxPoolClient {
            tx_count: 0,
        }
    }
}

impl TxPool for TxPoolClient {
    fn send_transaction(&self) -> Result<String> {
        Ok(format!("{}", self.tx_count))
    }
}