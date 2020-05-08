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

use std::marker::PhantomData;

use serde::{Serialize, Deserialize};
use bincode;
use hash;
use crate::types::{Hash, Address};
use crate::storage::{List, ListEntry};
use crate::state::{ArchiveDB, StateDB};


#[derive(Copy, Clone)]
enum StatePrefix {
    /// Validators list key
    Validator = 2,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct Validator {
    pub address: Address,
    pub pubkey: Vec<u8>,
    pub balance: u64,
    pub activate_height: u64,
}

impl Validator {
    pub fn map_key(&self) -> Hash {
        let mut raw = vec![];
        raw.extend_from_slice(Hash::from_bytes(self.address.as_slice()).as_bytes());
        let position = Hash::from_bytes(&(StatePrefix::Validator as u64).to_be_bytes()[..]);
        raw.extend_from_slice(position.as_bytes());

        Hash(hash::blake2b_256(&raw))
    }

    pub fn key_index(addr: &Address) -> Hash {
        let mut raw = vec![];
        raw.extend_from_slice(Hash::from_bytes(addr.as_slice()).as_bytes());
        let position = Hash::from_bytes(&(StatePrefix::Validator as u64).to_be_bytes()[..]);
        raw.extend_from_slice(position.as_bytes());

        Hash(hash::blake2b_256(&raw))
    }
}

pub struct Staking {
    pub validators: List<Validator>,
    pub state_db: StateDB,
}

impl Staking {
    pub fn from_state(backend: &ArchiveDB, root: Hash) -> Self {
        let head_key = Hash::from_bytes(&(StatePrefix::Validator as u64).to_be_bytes()[..]);
        Staking {
            validators: List::new(head_key),
            state_db: StateDB::from_existing(backend, root),
        }
    }

    pub fn insert(&mut self, item: &Validator) {
        let head = self.state_db.get_storage(&self.validators.head_key);
        if head.is_none() {
            let entry = ListEntry {
                pre: None,
                next: None,
                payload: item,
            };
            let encoded: Vec<u8> = bincode::serialize(&entry).unwrap();
            self.state_db.set_storage(item.map_key(), &encoded);
        } else {
            let head_ref = Hash::from_bytes(&head.unwrap()[..]);

            let entry = ListEntry {
                pre: None,
                next: Some(head_ref),
                payload: item,
            };
            self.state_db.set_storage(item.map_key(), &bincode::serialize(&entry).unwrap());
            {
                // replace next entry of inserted item
                let encoded = self.state_db.get_storage(&head_ref).unwrap();
                let mut next: ListEntry<Validator> = bincode::deserialize(&encoded).unwrap();
                next.pre = Some(item.map_key());
                let serialized: Vec<u8> = bincode::serialize(&next).unwrap();
                self.state_db.set_storage(next.payload.map_key(), &serialized);
            }
            // place reference of first item at head
            self.state_db.set_storage(self.validators.head_key, item.map_key().as_bytes());
        }
    }

    pub fn delete(&mut self, addr: &Address) {
        let encoded = match self.state_db.get_storage(&Validator::key_index(addr)) {
            Some(i) => i,
            None => return,
        };

        let item: ListEntry<Validator> = bincode::deserialize(&encoded).unwrap();
        if item.pre.is_some() {
            let encoded = self.state_db.get_storage(&item.pre.unwrap()).unwrap();
            let mut pre_node: ListEntry<Validator> = bincode::deserialize(&encoded).unwrap();
            pre_node.next = item.next;
            self.state_db.set_storage(item.pre.unwrap(), &bincode::serialize(&pre_node).unwrap());
        } else {
            self.state_db.set_storage(self.validators.head_key, item.next.unwrap().as_bytes());
        }

        if item.next.is_some() {
            let encoded = self.state_db.get_storage(&item.next.unwrap()).unwrap();
            let mut next_node: ListEntry<Validator> = bincode::deserialize(&encoded).unwrap();
            next_node.pre = item.pre;
            self.state_db.set_storage(item.next.unwrap(), &bincode::serialize(&next_node).unwrap());
        }

        // delete target from trie db
    }
    pub fn get_validator(&mut self, addr: &Address) -> Option<Validator> {
        // let head = self.state_db.get_storage(&self.validators.head_key);
        // if head.is_none() {
        //     return
        // }
        let encoded = match self.state_db.get_storage(&Validator::key_index(addr)) {
            Some(i) => i,
            None => return None,
        };

        let obj: ListEntry<Validator> = bincode::deserialize(&encoded).unwrap();
        Some(obj.payload)
    }

}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use env_logger;
    use map_store::{MemoryKV, KVDB};
    use crate::types::Address;
    use crate::state::ArchiveDB;
    use crate::trie::NULL_ROOT;
    use super::{Validator, Staking};

    #[test]
    fn validator_insert() {
        env_logger::init();
        let backend: Arc<RwLock<dyn KVDB>> = Arc::new(RwLock::new(MemoryKV::new()));
        let db = ArchiveDB::new(Arc::clone(&backend));
        let addr = Address::default();
        let first_addr = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();


        let validator = Validator {
            address: addr,
            pubkey: Vec::new(),
            balance: 1,
            activate_height: 1,
        };

        let mut stake = Staking::from_state(&db, NULL_ROOT);
        stake.insert(&validator);

        let first = Validator {
            address: first_addr,
            pubkey: Vec::new(),
            balance: 1,
            activate_height: 1,
        };

        stake.insert(&first);

        let item = stake.get_validator(&first_addr).unwrap();
        assert_eq!(item, first);
    }
}