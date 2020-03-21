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

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use bincode;
use hash;
use crate::types::{Hash, Address, H256};

const BALANCE_POS: u64 = 1;
const NONCE_POS: u64 = 2;

#[derive(Serialize, Deserialize)]
#[derive(Default, Copy, Clone)]
pub struct Account {
    // Available balance of eth account
    balance: u128,
    // Nonce of the account transaction count
    nonce: u64,
}

#[derive(Default)]
pub struct Balance {
    memdb: HashMap<Hash, Vec<u8>>,
}

impl Balance {

    pub fn new() -> Self {
        return Balance {
            memdb: HashMap::new(),
        }
    }

    pub fn balance(&self, addr: Address) -> u128 {
        let account = self.get_account(addr);
        account.balance
    }

    pub fn nonce(&self, addr: Address) -> u64 {
        let account = self.get_account(addr);
        account.nonce
    }

    #[allow(unused_variables)]
    pub fn transfer(&self, from_addr: Address, to_addr: Address, amount: u128) {
        let mut caller = self.get_account(from_addr);
        let mut receiver = self.get_account(to_addr);
        if caller.balance >= amount {
            caller.balance -= amount;
            receiver.balance += amount;
        }
    }

    pub fn get_account(&self, addr: Address) -> Account {
        let serialized = match self.memdb.get(&Self::address_key(addr)) {
            Some(s) => s,
            None => return Account::default(),
        };

        let obj: Account = bincode::deserialize(&serialized).unwrap();
        obj
    }

    pub fn set_account(&mut self, addr: Address, account: &Account) {
        let encoded: Vec<u8> = bincode::serialize(account).unwrap();
        self.memdb.insert(Self::address_key(addr), encoded);
    }

    /// Storage hash key of account
    pub fn address_key(addr: Address) -> Hash {
        let h = Hash::from_bytes(addr.as_slice());
        Hash(hash::blake2b_256(&h.to_slice()))
    }

    /// Storage hash key of account balance
    pub fn balance_key(addr: Address) -> Hash {
        let mut raw = vec![];
        {
            let h = Hash::from_bytes(addr.as_slice());
            raw.extend_from_slice(h.to_slice());
        }
        {
            let h = Hash::from_bytes(&BALANCE_POS.to_be_bytes()[..]);
            raw.extend_from_slice(h.to_slice());
        }
        Hash(hash::blake2b_256(&raw))
    }

    /// Storage hash key of account nonce
    pub fn nonce_key(addr: Address) -> Hash {
        let mut raw = vec![];
        {
            let h = Hash::from_bytes(addr.as_slice());
            raw.extend_from_slice(h.to_slice());
        }
        {
            let h = Hash::from_bytes(&NONCE_POS.to_be_bytes()[..]);
            raw.extend_from_slice(h.to_slice());
        }
        Hash(hash::blake2b_256(&raw))
    }
}