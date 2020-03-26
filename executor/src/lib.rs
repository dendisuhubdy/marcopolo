// Copyright 2019 MarcoPolo Protocol Authors.
// This file is part of MarcoPolo Protocol.

// MarcoPolo Protocol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// MarcoPolo Protocol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with MarcoPolo Protocol.  If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
extern crate log;
// use std::error::Error;
use core::transaction::Transaction;
use core::balance::Balance;
use core::types::{Hash, Address};
use core::block::{Block};

#[allow(non_upper_case_globals)]
const transfer_fee: u128 = 10000;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    InvalidSignData,
    BalanceNotEnough,
    InvalidTxNonce,
}
pub struct Executor;

impl Executor {
    pub fn exc_txs_in_block(b: &Block, state: &mut Balance, miner_addr: &Address) -> Result<Hash,Error> {
        let txs = b.get_txs();
        // let mut h = Hash([0u8;32]);
        for tx in txs {
            let res = Executor::exc_transfer_tx(tx,state);
            match res {
                Ok(v) => {},
                Err(e) => {
                    error!("Fail transfer exection {:?}", e);
                },
            };
        }

        let gas = transfer_fee * txs.len() as u128;
        state.add_balance(*miner_addr, gas);

        Ok(state.commit())
    }

    // handle the state for the tx,caller handle the gas of tx
    pub fn exc_transfer_tx(tx: &Transaction, state: &mut Balance) -> Result<Hash, Error> {
        // 1. version check
        // 2. nonce check
        // 3. balance check
        // 4. sign check
        // 5. update state
        let from_addr = tx.get_from_address();
        let to_addr = tx.get_to_address();

        // Ensure balance and nance field available
        let from_account = state.get_account(from_addr);
        if tx.get_nonce() != from_account.get_nonce() + 1 {
            return Err(Error::InvalidTxNonce);
        }
        if tx.get_value() + transfer_fee <= from_account.get_balance() {
            return Err(Error::BalanceNotEnough);
        }

        state.sub_balance(from_addr, transfer_fee);
        state.inc_nonce(from_addr);

        Executor::verify_tx_sign(&tx)?;
        state.transfer(from_addr, to_addr, tx.get_value());
        Ok(Hash::default())
    }

    // handle the state for the contract
    pub fn exc_contract_tx() -> Result<(),Error> {
        Ok(())
    }
    fn verify_tx_sign(tx: &Transaction) -> Result<(),Error> {
        if tx.verify_sign() {
            Ok(())
        } else {
            Err(Error::InvalidSignData)
        }
    }
}
