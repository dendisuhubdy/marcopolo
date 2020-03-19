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

use std::fmt;
use serde::{Serialize, Deserialize};
use ed25519::{H256, Message};
use ed25519::pubkey::Pubkey;
use hash;

#[derive(Serialize, Deserialize)]
#[derive(Default, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn to_slice(&self) -> &[u8] {
        return &self.0
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
    pub fn to_msg(&self) -> Message {
        H256(self.0)
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.0.iter() {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.0[..4].iter() {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Default, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub fn as_slice(&self) -> &[u8] {
        return &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.0.iter() {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl From<Pubkey> for Address {
    fn from(pk: Pubkey) -> Self {
        let raw = pk.to_bytes();
        let mut addr = Address::default();
        addr.0.copy_from_slice(&(hash::blake2b_256(&raw)[12..]));
        addr
    }
}