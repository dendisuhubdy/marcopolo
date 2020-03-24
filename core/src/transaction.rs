extern crate serde;
use serde::{Serialize, Deserialize};
use bytes::Bytes;
use super::types::{Address};

/// Represents a transaction
#[derive(Default, Debug, Clone, PartialEq, Eq,Serialize, Deserialize)]
pub struct Transaction {
	/// sender.
	pub sender: String,
	/// recipient.
	pub recipient: String,
	/// Nonce.
	pub nonce: u64,
	/// Gas price.
	pub gas_price: u64,
	/// Gas paid up front for transaction execution.
	pub gas: u64,
	/// Transfered value.
	pub value: u64,
	/// Transaction data.
	pub data: Bytes,
}

impl Transaction {
	pub fn get_to_address(&self) -> Address {
		Address([0u8;20])
	}
	pub fn get_from_address(&self) -> Address {
		Address([0u8;20])
	}
	pub fn get_nonce(&self) -> u64 {
		self.nonce
	}
	pub fn get_value(&self) -> u64 {
		self.value
	}
}
