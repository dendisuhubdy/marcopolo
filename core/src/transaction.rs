extern crate serde;
extern crate errors;

#[allow(unused_imports)]
use errors::{Error,InternalErrorKind};

use super::types::{Address};
use ed25519::{signature::SignatureInfo,privkey::PrivKey,pubkey::Pubkey};
use serde::{Deserialize, Serialize};
use bincode;

use super::types::{Hash,chain_id};

/// Message call identifer length
pub const MSGID_LENGTH: usize = 4;

/// Represents a transaction
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Hash, Deserialize)]
pub struct Transaction {
	/// sender.
	pub sender: Address,
	/// Nonce.
	pub nonce: u64,
	/// Gas price.
	pub gas_price: u64,
	/// Gas paid up front for transaction execution.
	pub gas: u64,
    /// Message function call
    pub method: [u8; MSGID_LENGTH],
	/// Transaction message data
	pub data: Vec<u8>,
	pub sign_data: ([u8;32],[u8;32],[u8;32]),
}

pub mod balance_msg {
    use serde::{Deserialize, Serialize};
    use crate::types::{Address};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Transfer {
        pub receiver: Address,
        pub value: u128,
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct tx_hash_type {
	chainid: 	u32,
	nonce: u64,
	gas_price: u64,
	gas: u64,
	method: [u8; MSGID_LENGTH],
	data: Vec<u8>,
}

impl tx_hash_type {
	fn new(tx: &Transaction) -> Self {
		tx_hash_type{
			chainid:chain_id,
			nonce: tx.nonce,
			gas_price: tx.gas_price,
			gas: tx.gas,
			method: tx.method,
			data: tx.data.clone(),
		}
	}
}

impl Transaction {
	pub fn get_to_address(&self) -> Address {
        let input: balance_msg::Transfer = bincode::deserialize(&self.data).unwrap();
        input.receiver
	}
	pub fn get_from_address(&self) -> Address {
        self.sender
	}
	pub fn get_nonce(&self) -> u64 {
		self.nonce
	}
	pub fn get_value(&self) -> u128 {
        let input: balance_msg::Transfer = bincode::deserialize(&self.data).unwrap();
        input.value
	}
	pub fn get_sign_data(&self) -> SignatureInfo {
		SignatureInfo::make(self.sign_data.0,self.sign_data.1,self.sign_data.2)
	}
	pub fn new(sender: Address, nonce: u64, gas_price: u64, gas: u64,
		method: [u8; MSGID_LENGTH], data: Vec<u8>) -> Transaction {
        Transaction {
           sender: sender,
            nonce:nonce,
            gas_price:gas_price,
            gas:gas,
            sign_data: ([0u8;32],[0u8;32],[0u8;32]),
            method: method,
            data:data,
        }
    }

	pub fn hash(&self) -> Hash {
		let data = tx_hash_type::new(self);
		let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
        Hash(hash::blake2b_256(encoded))
	}
	fn set_sign_data(&mut self,data: &SignatureInfo) {
		self.sign_data.0[..].copy_from_slice(data.r());
		self.sign_data.1[..].copy_from_slice(data.s());
		self.sign_data.2[..].copy_from_slice(data.p());
	}
	pub fn sign(&mut self,priv_data: &[u8]) -> Result<(),Error> {
		let h = self.hash();
		let priv_key = PrivKey::from_bytes(priv_data);
		let data = priv_key.sign(h.to_slice())?;
		self.set_sign_data(&data);
		Ok(())
	}
	pub fn verify_sign(&self) -> Result<(),Error> {
		let pk = Pubkey::from_bytes(&self.sign_data.2[..]);
		pk.verify(&self.hash().to_msg(), &self.get_sign_data())
	}
}

#[cfg(test)]
mod tests {
    use bincode;
    use super::*;

    #[test]
    fn unpack_transfer() {
        let msg = balance_msg::Transfer {
            receiver: Address::default(),
            value: 1,
        };
        let encoded: Vec<u8> = bincode::serialize(&msg).unwrap();
        let tx: balance_msg::Transfer = bincode::deserialize(&encoded).unwrap();
        assert_eq!(tx.value, 1);
    }
}