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

use crate::types::{Hash, Address};
use crate::storage::List;
use crate::state::{ArchiveDB, StateDB};


#[derive(Copy, Clone)]
enum StatePrefix {
    /// Validators list key
    Validator = 2,
}

pub struct Validator {
    pub address: Address,
    pub pubkey: Vec<u8>,
    pub balance: u64,
    pub activate_height: u64,
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
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use env_logger;
    use map_store::{MemoryKV, KVDB};
    use crate::types::Address;
    use crate::state::ArchiveDB;

    #[test]
    fn validator_insert() {
        env_logger::init();
        let backend: Arc<RwLock<dyn KVDB>> = Arc::new(RwLock::new(MemoryKV::new()));
        let db = ArchiveDB::new(Arc::clone(&backend));
    }
}