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
extern crate enum_display_derive;

use failure::{Backtrace, Context, Fail};
use std::fmt::{self, Display};

pub mod poa;
pub mod traits;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Display)]
pub enum ErrorKind {
    Header,
    Block,
    Verify,
}

#[derive(Debug)]
pub struct Error {
    kind: Context<ErrorKind>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(cause) = self.cause() {
            if f.alternate() {
                write!(f, "{}: {}", self.kind(), cause)
            } else {
                write!(f, "{}({})", self.kind(), cause)
            }
        } else {
            write!(f, "{}", self.kind())
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Self { kind: inner }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.kind.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.kind.backtrace()
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.kind.get_context()
    }

    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        self.cause().and_then(|cause| cause.downcast_ref::<T>())
    }
}
