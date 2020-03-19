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
