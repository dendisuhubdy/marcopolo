extern crate serde;

use std::fmt;

use bytes::Bytes;
use super::types::{Address};
use ed25519::{signature::SignatureInfo,Message};
use serde::{Deserialize, Serialize};

use super::types::Hash;

/// Represents a transaction
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
	/// sender.
	pub sender: Address,
	/// recipient.
	pub recipient: Address,
	/// Nonce.
	pub nonce: u64,
	/// Gas price.
	pub gas_price: u64,
	/// Gas paid up front for transaction execution.
	pub gas: u64,
	/// Transfered value.
	pub value: u128,
	pub sign_data: ([u8;32],[u8;32],[u8;32]),
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
	pub fn get_value(&self) -> u128 {
		self.value
	}
	pub fn get_sign_data(&self) -> SignatureInfo {
		SignatureInfo::make(self.sign_data.0,self.sign_data.1,self.sign_data.2)
	}
	pub fn new(sender: Address, recipient: Address, nonce: u64, gas_price: u64, gas: u64, 
		value: u128, data: Bytes) -> Transaction {
        Transaction {
           sender: sender,
            recipient:recipient,
            nonce:nonce,
            gas_price:gas_price,
            gas:gas,
            value:value,
            sign_data: ([0u8;32],[0u8;32],[0u8;32]),
            data:data,
        }
    }

    pub fn hash(&self) -> Hash {
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        Hash(hash::blake2b_256(encoded))
    }
}
