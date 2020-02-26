extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use bytes::Bytes;

/// Represents a transaction
#[derive(Default, Debug, Clone, PartialEq, Eq)]
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
