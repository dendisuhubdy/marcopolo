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
#[macro_use]
extern crate log;

use failure::{Backtrace,err_msg, Context, Fail};
use std::fmt::{self, Display,Debug};

pub mod poa;
pub mod traits;


#[derive(Debug, Clone, Copy, Eq, PartialEq, Display)]
pub enum ErrorKind {
    Header,
    Block,
    Internal,
    Consensus,
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

//////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct ConsensusError {
    kind: Context<ConsensusErrorKind>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Display)]
pub enum ConsensusErrorKind {
    Header,
    Block,
    Verify,
    NoneSign,
    Execute,
}

impl fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(cause) = self.cause() {
            write!(f, "{}({})", self.kind(), cause)
        } else {
            write!(f, "{}", self.kind())
        }
    }
}

impl From<ConsensusError> for Error {
    fn from(error: ConsensusError) -> Self {
        error.context(ErrorKind::Internal).into()
    }
}

impl From<ConsensusErrorKind> for ConsensusError {
    fn from(kind: ConsensusErrorKind) -> Self {
        ConsensusError {
            kind: Context::new(kind),
        }
    }
}

impl From<ConsensusErrorKind> for Error {
    fn from(kind: ConsensusErrorKind) -> Self {
        Into::<ConsensusError>::into(kind).into()
    }
}

impl ConsensusErrorKind {
    pub fn cause<F: Fail>(self, cause: F) -> ConsensusError {
        ConsensusError {
            kind: cause.context(self),
        }
    }

    pub fn reason<S: Display + Debug + Sync + Send + 'static>(self, reason: S) -> ConsensusError {
        ConsensusError {
            kind: err_msg(reason).compat().context(self),
        }
    }
}

impl ConsensusError {
    pub fn kind(&self) -> &ConsensusErrorKind {
        &self.kind.get_context()
    }
}

impl Fail for ConsensusError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.kind.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.kind.backtrace()
    }
}
