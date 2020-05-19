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

use std::cell::RefCell;
use std::rc::Rc;

use crate::state::{StateDB};
use crate::staking::{Staking, Validator};
use crate::balance::{Balance, Account};
use crate::types::Address;

pub struct Interpreter {
    state_db: Rc<RefCell<StateDB>>,
}

impl Interpreter {
    pub fn new(backend: Rc<RefCell<StateDB>>) -> Self {
        Interpreter {
            state_db: backend.clone(),
        }
    }

    pub fn call(&mut self, caller: &Address, msg: Vec<u8>, input: Vec<u8>) {
        let sep = msg.iter().position(|&x| x == '.' as u8);
        if sep.is_none() {
            return
        }
        let (module, func) = msg.split_at(sep.unwrap());

        if module == b"balance" {
            // Balance::from_state(self.state_db.clone()).transfer(calller, input);
        } else if module == b"staking" {
            let mut state = Staking::from_state(self.state_db.clone());
            match func {
                b"validate" => state.exec_validate(caller, input),
                b"deposit" => state.exec_deposit(caller, input),
                _ => warn!("invalid staking call"),
            }
        } else {
            warn!("unsupport msg call");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use std::rc::Rc;
    use std::cell::RefCell;
    use bincode;
    use map_store::{MemoryKV, KVDB};
    use crate::state::{ArchiveDB, StateDB};
    use crate::types::Address;
    use crate::trie::NULL_ROOT;
    use super::{Interpreter};

    #[test]
    fn interpreter_call() {
        let backend: Arc<RwLock<dyn KVDB>> = Arc::new(RwLock::new(MemoryKV::new()));
        let db = ArchiveDB::new(Arc::clone(&backend));
        let state_db = Rc::new(RefCell::new(StateDB::from_existing(&db, NULL_ROOT)));
        let mut runner = Interpreter::new(state_db.clone());
        runner.call(&Address::default(), b"staking.deposit".to_vec(), bincode::serialize(&1u128).unwrap());
    }
}
