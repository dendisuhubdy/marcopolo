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
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use bincode;
use hash;
use crate::types::{Hash, Address};
use crate::state::{ArchiveDB, StateDB};

const BALANCE_POS: u64 = 1;
const NONCE_POS: u64 = 2;

#[derive(Serialize, Deserialize)]
#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Account {
    // Available balance of eth account
    balance: u128,
    // Nonce of the account transaction count
    nonce: u64,
}

impl Account {
    pub fn get_balance(&self) -> u128 {
        self.balance
    }
    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }
}

pub struct Balance {
    cache: HashMap<Hash, Account>,
    treedb: StateDB,
    root_hash: Hash,
}

impl Balance {

    pub fn new(backend: &ArchiveDB) -> Self {
        Balance {
            cache: HashMap::new(),
            treedb: StateDB::new(backend),
            root_hash: Hash::default(),
        }
    }

    pub fn from_state(backend: &ArchiveDB, root: Hash) -> Self {
        Balance {
            cache: HashMap::new(),
            treedb: StateDB::from_existing(backend, root),
            root_hash: root,
        }
    }

    pub fn balance(&self, addr: Address) -> u128 {
        let addr_hash = Self::address_key(addr);
        let account = match self.cache.get(&addr_hash) {
            Some(v) => v.clone(),
            None => self.load_account(addr),
        };
        account.balance
    }

    pub fn nonce(&self, addr: Address) -> u64 {
        let addr_hash = Self::address_key(addr);
        let account = match self.cache.get(&addr_hash) {
            Some(v) => v.clone(),
            None => self.load_account(addr),
        };
        account.nonce
    }

    pub fn get_account(&self, addr: Address) -> Account {
        let addr_hash = Self::address_key(addr);
        let account = match self.cache.get(&addr_hash) {
            Some(v) => v.clone(),
            None => self.load_account(addr),
        };
        account
    }

    pub fn inc_nonce(&mut self, addr: Address) {
        let addr_hash = Self::address_key(addr);
        let mut account = match self.cache.get(&addr_hash) {
            Some(v) => v.clone(),
            None => self.load_account(addr),
        };
        account.nonce += 1;
        self.cache.insert(addr_hash, account);
    }

    pub fn add_balance(&mut self, addr: Address, value: u128) {
        let addr_hash = Self::address_key(addr);
        let mut account = match self.cache.get(&addr_hash) {
            Some(v) => v.clone(),
            None => self.load_account(addr),
        };
        account.balance += value;
        self.cache.insert(addr_hash, account);
    }

    pub fn sub_balance(&mut self, addr: Address, value: u128) {
        let addr_hash = Self::address_key(addr);
        let mut account = match self.cache.get(&addr_hash) {
            Some(v) => v.clone(),
            None => self.load_account(addr),
        };
        account.balance -= value;
        self.cache.insert(addr_hash, account);
    }

    pub fn reset(&mut self) {
        self.cache.clear();
    }

    pub fn transfer(&mut self, from_addr: Address, to_addr: Address, amount: u128) {
        if self.balance(from_addr) >= amount {
            self.sub_balance(from_addr, amount);
            self.add_balance(to_addr, amount);
        } else {
            // take transaction fee
        }
    }

    pub fn commit(&mut self) -> Hash {
        for (addr_hash, account) in self.cache.iter() {
            let encoded: Vec<u8> = bincode::serialize(&account).unwrap();
            self.treedb.set_storage(*addr_hash, &encoded);
        }
        self.treedb.commit();
        self.cache.clear();
        self.root_hash = self.treedb.root();
        self.root_hash
    }

    pub fn load_account(&self, addr: Address) -> Account {
        // let serialized = match self.cache.get(&Self::address_key(addr)) {
        //     Some(s) => s,
        //     None => return Account::default(),
        // };
        let serialized = match self.treedb.get_storage(&Self::address_key(addr)) {
            Some(s) => s,
            None => return Account::default(),
        };

        let obj: Account = bincode::deserialize(&serialized).unwrap();
        obj
    }

    pub fn set_account(&mut self, addr: Address, account: &Account) {
        let encoded: Vec<u8> = bincode::serialize(account).unwrap();
        self.treedb.set_storage(Self::address_key(addr), &encoded);
    }

    pub fn load_root(&mut self, root: Hash) {
        self.root_hash = root;
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

mod tests {
    use super::*;

    #[test]
    fn test_set_account() {
        let addr = Address::default();
        let mut state = Balance::new(PathBuf::from("."));
        let mut account = state.load_account(addr);
        assert_eq!(account, Account::default());

        let v1 = Account {
            balance: 1,
            nonce: 1
        };
        state.set_account(addr, &v1);
        account = state.load_account(addr);
        assert_eq!(account, v1);
    }

    #[test]
    fn test_change_accounts() {
        let from = Address([0; 20]);
        let to = Address([1; 20]);
        let mut state = Balance::new(PathBuf::from("."));
        state.set_account(from, &Account {
            balance: 1,
            nonce: 1,
        });
        state.set_account(to, &Account {
            balance: 2,
            nonce: 1,
        });
        let account = state.load_account(from);
        assert_eq!(account.balance, 1);
    }

    #[test]
    fn test_transfer() {
        let mut state = Balance::new(PathBuf::from("."));
        let addr = Address::default();
        state.set_account(addr, &Account {
            balance: 1,
            nonce: 1,
        });

        let receiver = Address([1; 20]);
        state.set_account(receiver, &Account {
            balance: 0,
            nonce: 0,
        });

        state.transfer(addr, receiver, 1);
        state.commit();
        let account = state.load_account(receiver);
        assert_eq!(account.balance, 1);
        assert_eq!(state.balance(receiver), 1);
    }
}
