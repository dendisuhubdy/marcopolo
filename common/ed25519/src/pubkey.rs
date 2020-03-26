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
extern crate bincode;
extern crate sha2;
extern crate errors;

use errors::{Error,InternalErrorKind};
use bincode::{serialize};
use ed25519_dalek::{PublicKey,Signature};
use super::signature::SignatureInfo;
use super::{H256,Message};
use sha2::Sha512;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Pubkey {
    inner: H256,
}

impl Pubkey {

    pub fn to_bytes(&self)->Vec<u8> {
        Vec::from(&self.inner.0[..])
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut pk: [u8; 32] = [0u8; 32];
        pk.copy_from_slice(&bytes[..32]);
        Pubkey{inner: H256(pk)}
    }
    pub fn from_pubkey(pk: &PublicKey) -> Self {
        Pubkey{inner: H256(pk.to_bytes())}
    }
    #[inline]
    pub fn to_pubkey(&self)->Result<PublicKey,Error> {
        PublicKey::from_bytes(&self.inner.0[..])
        .map_err(|e|InternalErrorKind::Other(e.to_string()).into())
    }
    #[inline]
    pub fn verify(&self, message: &Message, signinfo: &SignatureInfo) -> Result<(),Error> {
        let sign: Signature = signinfo.to_signature().unwrap();
        let pubkey: PublicKey = self.to_pubkey()?;
        PublicKey::verify::<Sha512>(&pubkey,&message.0,&sign)
        .map_err(|e|InternalErrorKind::Other(e.to_string()).into())
    }
}

#[cfg(test)]
pub mod tests {
    extern crate errors;
    use errors::{Error,InternalErrorKind};
    use super::{H256,Message,Pubkey};

    #[test]
    pub fn test_error_handle() -> Result<(),Error> {
        let pk = Pubkey{inner: H256([0u8;32])};
        let res = pk.to_pubkey();
        match res {
            Ok(p) => {
                println!("ok....");
                Ok(())
            },
            Err(e) => Err(InternalErrorKind::Other(e.to_string()).into()),
        }
    }
}

