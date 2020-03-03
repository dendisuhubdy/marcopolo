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
use ed25519_dalek::{Signature,SignatureError};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, KEYPAIR_LENGTH, SIGNATURE_LENGTH};

#[derive(Clone)]
pub struct SignatureInfo([u8; SIGNATURE_LENGTH]);

impl SignatureInfo {

    /// Get a slice into the 'r' portion of the data.
    pub fn r(&self) -> &[u8] {
        &self.0[0..32]
    }

    /// Get a slice into the 's' portion of the data.
    pub fn s(&self) -> &[u8] {
        &self.0[32..64]
    }

    pub fn to_signature(&self) -> Result<Signature, SignatureError> {
        let data = &self.0;
        let res = Signature::from_bytes(data);
        res
    }
    pub fn from_signature(sign: &Signature) -> Self {
        let data = sign.to_bytes();
        SignatureInfo(data)
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, SignatureError> {
        // let mut sig = [0u8; SIGNATURE_LENGTH];
        // sig[..].copy_from_slice(data);
        let sig: Signature = Signature::from_bytes(data).unwrap();
        Ok(SignatureInfo::from_signature(&sig))
    }
}