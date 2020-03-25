extern crate serde;

use std::fmt;

use bytes::Bytes;
use super::types::{Address};
use ed25519::{signature::SignatureInfo,Message,privkey::PrivKey,pubkey::Pubkey};
use serde::{Deserialize, Serialize};

use super::types::{Hash,chain_id};

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
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct tx_hash_type {
	chainid: 	u32,
	recipient: Address,
	nonce: u64,
	gas_price: u64,
	gas: u64,
	value: u128,
	data: Bytes,
}
impl tx_hash_type {
	fn new(tx: &Transaction) -> Self {
		tx_hash_type{
			chainid:chain_id,
			recipient: tx.recipient,
			nonce: tx.nonce,
			gas_price: tx.gas_price,
			gas: tx.gas,
			value: tx.value,
			data: tx.data.clone(),
		}
	}
}

impl Transaction {
	pub fn get_to_address(&self) -> Address {
		self.sender
	}
	pub fn get_from_address(&self) -> Address {
		self.recipient
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
		let data = tx_hash_type::new(self);
		let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
        Hash(hash::blake2b_256(encoded))
	}
	fn set_sign_data(&mut self,data: &SignatureInfo) {
		self.sign_data.0[..].copy_from_slice(data.r());
		self.sign_data.1[..].copy_from_slice(data.s());
		self.sign_data.2[..].copy_from_slice(data.p());
	}
	pub fn sign(&mut self,priv_data: &[u8]) {
		let h = self.hash();
		let priv_key = PrivKey::from_bytes(priv_data);
		let data = priv_key.sign(h.to_slice());
		self.set_sign_data(&data);
	}
	pub fn verify_sign(&self) -> bool {
		let pk = Pubkey::from_bytes(&self.sign_data.2[..]);
		if pk.verify(&self.hash().to_msg(), &self.get_sign_data()).is_err() {
			return  false;
		}
		return true;
	}
}
