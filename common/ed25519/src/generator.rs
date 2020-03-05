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

extern crate rand_os;
extern crate ed25519_dalek;

use rand_os::OsRng;
use ed25519_dalek::{PublicKey,SecretKey,Signature,SignatureError};
use super::{privkey::PrivKey,pubkey::Pubkey};

pub struct Generator {}

impl Default for Generator {
    fn default() -> Self {
        Generator{}
    }
}

impl Generator {
    pub fn new(&self) -> (PrivKey,Pubkey) {
        let mut csprng: OsRng = OsRng::new().unwrap();
        let sk: SecretKey = SecretKey::generate(&mut csprng);
        let priv_key: PrivKey = PrivKey::from_secret_key(&sk);
        (priv_key,priv_key.to_pubkey())
    }
}

#[test]
fn generatePair() {
    println!("start generatePair test....");
    let (priv_key,pub_key) = Generator::default().new();
    println!("priv_key:{:?},pub_key:{:?}",priv_key,pub_key);
    println!("end generatePair test....");
}
