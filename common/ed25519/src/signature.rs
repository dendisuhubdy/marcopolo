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

use serde::{Serialize, Deserialize};
use ed25519_dalek::{Signature,SignatureError};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, KEYPAIR_LENGTH, SIGNATURE_LENGTH};
use std::fmt;
use faster_hex::hex_string;
use std::fmt::Error;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Default,Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct SignatureInfo([u8; 32], [u8;32]);

impl SignatureInfo {
    pub fn r(&self) -> &[u8] {
        &self.0[..]
    }
    pub fn s(&self) -> &[u8] {
        &self.1[..]
    }
    pub fn make(r:[u8;32],s:[u8;32]) -> Self {
        SignatureInfo(r,s)
    }
    pub fn to_signature(&self) -> Result<Signature, SignatureError> {
        let mut sig = [0u8; 64];
        sig[0..32].copy_from_slice(&self.0[..]);
        sig[32..64].copy_from_slice(&self.1[..]);
        let res = Signature::from_bytes(&sig);
        res
    }
    pub fn from_signature(sign: &Signature) -> Self {
        let data = sign.to_bytes();
        let mut r = [0u8;32];
        let mut s = [0u8;32];
        r[..].copy_from_slice(&data[0..32]);
        s[..].copy_from_slice(&data[32..64]);
        SignatureInfo(r,s)
    }
    pub fn from_slice(data: &[u8]) -> Result<Self, SignatureError> {
        // let mut sig = [0u8; SIGNATURE_LENGTH];
        // sig[..].copy_from_slice(data);
        let sig: Signature = Signature::from_bytes(data).unwrap();
        Ok(SignatureInfo::from_signature(&sig))
    }
}

// #[derive(Clone)]
// pub struct SignatureInfo(pub [u8; SIGNATURE_LENGTH]);

// impl Default for SignatureInfo {
//     fn default() -> Self {
//         SignatureInfo([0u8;SIGNATURE_LENGTH])
//     }
// }
// impl fmt::Debug for SignatureInfo {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         f.debug_struct("Signature")
//             .field("r", &hex_string(&self.0[0..32]).expect("hex string"))
//             .field("s", &hex_string(&self.0[32..64]).expect("hex string"))
//             .field("v", &hex_string(&self.0[64..65]).expect("hex string"))
//             .finish()
//     }
// }

// impl SignatureInfo {

//     /// Get a slice into the 'r' portion of the data.
//     pub fn r(&self) -> &[u8] {
//         &self.0[0..32]
//     }

//     /// Get a slice into the 's' portion of the data.
//     pub fn s(&self) -> &[u8] {
//         &self.0[32..64]
//     }

//     pub fn to_signature(&self) -> Result<Signature, SignatureError> {
//         let data = &self.0;
//         let res = Signature::from_bytes(data);
//         res
//     }
//     pub fn from_signature(sign: &Signature) -> Self {
//         let data = sign.to_bytes();
//         SignatureInfo(data)
//     }

//     pub fn from_slice(data: &[u8]) -> Result<Self, SignatureError> {
//         // let mut sig = [0u8; SIGNATURE_LENGTH];
//         // sig[..].copy_from_slice(data);
//         let sig: Signature = Signature::from_bytes(data).unwrap();
//         Ok(SignatureInfo::from_signature(&sig))
//     }
// }