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

//! MarcoPolo ED25519.

extern crate ed25519_dalek;
extern crate serde;
extern crate bincode;

use bincode::{serialize, Infinite};
use ed25519_dalek::{PublicKey,Signature,SignatureError};
use super::signature::SignatureInfo;
use crate::hash::H256;


#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Pubkey {
    inner: H256,
}

impl Pubkey {

    pub fn to_bytes(&self) -> Vec<u8> {
        Vec::from(&self.inner.0[..])
    }

    pub fn to_pubkey(&self) -> Result(PublicKey,SignatureError) {
        PublicKey::from_bytes(&self.inner.0[..])
    }

    pub fn verify(&self, message: &Message, signinfo: &SignatureInfo) -> Result<(), SignatureError> {
        let sign: Signature = signinfo.to_signature().unwrap();
        let pubkey: PublicKey = to_pubkey.to_pubkey().unwrap();
        pubkey.verify(&message.0,sign)?;
        Ok(())
    }
}
