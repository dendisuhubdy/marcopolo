use bytes::Bytes;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use ed25519::{privkey::PrivKey, pubkey::Pubkey};
use map_core::genesis::{ed_genesis_priv_key, ed_genesis_pub_key};
use map_core::transaction::Transaction;
use map_core::types::Address;

/// TxPool rpc interface.
#[rpc(server)]
pub trait TxPool {
    /// Returns protocol version.
    #[rpc(name = "map_sendTransaction")]
    fn send_transaction(&self, from: String, to: String, value: u128) -> Result<String>;
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
    fn send_transaction(&self, from: String, to: String, value: u128) -> Result<String> {
        if is_hex(from.as_str()).is_ok() {
            return Ok(format!("from address is not hex {}", from));
        }
        if is_hex(to.as_str()).is_ok() {
            return Ok(format!("to address is not hex {}", to));
        }

        let from = match Address::from_hex(&from) {
            Ok(v) => v,
            Err(e) => return Ok(format!("convert address err  {} {}", &from, e))
        };

        let to = match Address::from_hex(&to) {
            Ok(v) => v,
            Err(e) => return Ok(format!("convert address err  {} {}", &to, e))
        };

        let b = Bytes::new();
        let pkey = PrivKey::from_bytes(&ed_genesis_priv_key);
        let pk = Pubkey::from_bytes(&ed_genesis_pub_key);
        let sign_address = Address::from(pk);

        if sign_address != from {
            return Ok(format!("sign address error  {} {}", sign_address, from));
        }

        let mut tx = Transaction::new(sign_address, to, 1, 1000, 1000, value, b);

        tx.sign(&pkey.to_bytes());
        // self.txs.push(tx.clone());
        Ok(format!("{}", tx.hash()))
    }
}

fn is_hex(hex: &str) -> core::result::Result<(), String> {
    let tmp = hex.as_bytes();
    if tmp.len() < 2 {
        Err("Must be a 0x-prefix hex string".to_string())
    } else if tmp.len() & 1 != 0 {
        Err("Hex strings must be of even length".to_string())
    } else if tmp[..2] == b"0x"[..] {
        for byte in &tmp[2..] {
            match byte {
                b'A'..=b'F' | b'a'..=b'f' | b'0'..=b'9' => continue,
                invalid_char => {
                    return Err(format!("Hex has invalid char: {}", invalid_char));
                }
            }
        }
        Ok(())
    } else {
        Err("Must 0x-prefix hex string".to_string())
    }
}