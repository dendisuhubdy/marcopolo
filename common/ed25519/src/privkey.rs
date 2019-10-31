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

use ed25519_dalek::{SecretKey,Signature,PublicKey,SignatureError};.
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};
use crate::hash::H256;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PrivKey {
    inner: H256,
}

impl PrivKey {

    pub fn to_bytes(&self) -> [u8; SECRET_KEY_LENGTH] {
        self.inner.0
    }

    pub fn to_secrit_key(&self) -> Result<SecretKey, SignatureError> {
        let data = self.inner.0;
        let res = SecretKey::from_bytes(&data[..]);
        res
    }

    pub fn from_secret_key(key &SecretKey) -> Self {
        PrivKey(SecretKey::to_bytes())
    }

    pub fn sign(&self,message: &[u8]) -> Result<Signature, SignatureError>  {
        let sk: SecretKey = self.to_secrit_key().unwrap();
        let expanded_secret: ExpandedSecretKey = (&sk).into();
        let pk: PublicKey = self.to_pubkey().unwrap();
        expanded_secret.sign(&message,&pk)
    }

    pub fn to_pubkey(&self) -> Result<Pubkey,SignatureError> {
        let sk: SecretKey = self.to_secrit_key().unwrap();
        let public_from_secret: PublicKey = (&sk).into();
        OK(public_from_secret)
    }

}