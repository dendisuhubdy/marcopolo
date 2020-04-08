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

use hash as core_hash;
use std::hash;
use std::marker::PhantomData;
use hash_db;
use trie_db;
use trie_db::{DBValue, TrieLayout, NodeCodec, ChildReference, Partial,
    node::{NibbleSlicePlan, NodePlan, NodeHandlePlan}};
use serde::{Serialize, Deserialize};
use bincode;
use crate::types::{Hash, Address};

/// Hasher that just takes 8 bytes of the provided value.
/// May only be used for keys which are 32 bytes.
#[derive(Default)]
pub struct Hash256StdHasher {
    prefix: u64,
}

impl hash::Hasher for Hash256StdHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.prefix
    }

    #[inline]
    #[allow(unused_assignments)]
    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() < 32 { return }

        let mut bytes_ptr = bytes.as_ptr();
        let mut prefix_ptr = &mut self.prefix as *mut u64 as *mut u8;

        unroll! {
            for _i in 0..8 {
                unsafe {
                    *prefix_ptr ^= (*bytes_ptr ^ *bytes_ptr.offset(8)) ^ (*bytes_ptr.offset(16) ^ *bytes_ptr.offset(24));
                    bytes_ptr = bytes_ptr.offset(1);
                    prefix_ptr = prefix_ptr.offset(1);
                }
            }
        }
    }
}

const HASHED_NULL_NODE_BYTES :[u8;32] = [0;32];
const HASHED_NULL_NODE : Hash = Hash(HASHED_NULL_NODE_BYTES);

const EMPTY_TRIE: u8 = 0;
const NIBBLE_SIZE_BOUND: usize = u16::max_value() as usize;
const LEAF_PREFIX_MASK: u8 = 0b_01 << 6;
const BRANCH_WITHOUT_MASK: u8 = 0b_10 << 6;
const BRANCH_WITH_MASK: u8 = 0b_11 << 6;


#[derive(Default, Debug, Clone, PartialEq)]
pub struct Blake2Hasher;

impl hash_db::Hasher for Blake2Hasher {
    type Out = Hash;
    type StdHasher = Hash256StdHasher;
    const LENGTH: usize = 32;

    fn hash(x: &[u8]) -> Self::Out {
        let mut out = [0;32];
        out = core_hash::blake2b_256(x);
        out.into()
    }
}

/// implementation of a `NodeCodec`.
#[derive(Default, Clone)]
pub struct BinNodeCodec<H>(PhantomData<H>);

/// layout using modified partricia trie with extention node
pub struct ExtensionLayout;

impl TrieLayout for ExtensionLayout {
    const USE_EXTENSION: bool = true;
    type Hash = Blake2Hasher;
    type Codec = BinNodeCodec<Blake2Hasher>;
}

/// Encode a partial value with a partial tuple as input.
fn encode_partial_iter<'a>(partial: Partial<'a>, is_leaf: bool) -> impl Iterator<Item = u8> + 'a {
    encode_partial_inner_iter((partial.0).1, partial.1.iter().map(|v| *v), (partial.0).0 > 0, is_leaf)
}

/// Encode a partial value with an iterator as input.
fn encode_partial_from_iterator_iter<'a>(
    mut partial: impl Iterator<Item = u8> + 'a,
    odd: bool,
    is_leaf: bool,
) -> impl Iterator<Item = u8> + 'a {
    let first = if odd { partial.next().unwrap_or(0) } else { 0 };
    encode_partial_inner_iter(first, partial, odd, is_leaf)
}

/// Encode a partial value with an iterator as input.
fn encode_partial_inner_iter<'a>(
    first_byte: u8,
    partial_remaining: impl Iterator<Item = u8> + 'a,
    odd: bool,
    is_leaf: bool,
) -> impl Iterator<Item = u8> + 'a {
    let encoded_type = if is_leaf {0x20} else {0};
    let first = if odd {
        0x10 + encoded_type + first_byte
    } else {
        encoded_type
    };
    std::iter::once(first).chain(partial_remaining)
}

// extention node or leaf node
#[derive(Serialize, Deserialize)]
struct ShortNode(&[u8], &[u8]);

// branch node
#[derive(Serialize, Deserialize)]
struct FullNode(&[Option<&[u8]>], Option<&[u8]>);


impl<H: hash_db::Hasher> NodeCodec for BinNodeCodec<H> {
    type Error = CodecError;
    type HashOut = H::Out;

    fn hashed_null_node() -> <H as Hasher>::Out {
        H::hash(<Self as NodeCodec>::empty_node())
    }

    fn decode_plan(data: &[u8]) -> ::std::result::Result<NodePlan, Self::Error> {
        if let Ok(node) = bincode::deserialize::<ShortNode>(data) {
            let partial_header = node.0[0];
            let is_leaf = partial_header & 32 == 32;
            if is_leaf {
                Ok(NodePlan::Leaf { node.0, node.1 })
            } else {
                Ok(NodePlan::Extension { node.0, node.1 })
            }

        } else if let Ok(node) = bincode::deserialize::<FullNode>(data) {
            let mut children = [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ];
            for (i, child) in children.iter_mut().enumerate() {
                if node.0[i] != None {
                    *child = Some(node.1);
                }
            }
            let value = Some(node.1);
            Ok(NodePlan::Branch { value, children })

        } else {

        }
    }

    fn is_empty_node(data: &[u8]) -> bool {
        data == <Self as NodeCodec>::empty_node()
    }

    fn empty_node() -> &'static[u8] {
        &[EMPTY_TRIE]
    }

    fn leaf_node(partial: Partial, value: &[u8]) -> Vec<u8> {
        let mut path = Vec<u8>::new();
        path.extend(encode_partial_iter(partial, true));
        let node = ShortNode(&path, value);
        let encoded: Vec<u8> = bincode::serialize(&node).unwrap();
        encoded
    }

    fn extension_node(
        partial: impl Iterator<Item = u8>,
        number_nibble: usize,
        child: ChildReference<Self::HashOut>,
    ) -> Vec<u8> {
        let mut path = Vec<u8>::new();
        path.extend(encode_partial_iter(partial, true));
        let value = match child {
            ChildReference::Hash(hash) => &hash.0,
            ChildReference::Inline(inline_data, length) => &inline_data.0,
        };
        let node = ShortNode(&path, &value);
        let encoded: Vec<u8> = bincode::serialize(&node).unwrap();
        encoded
    }

    fn branch_node(
        children: impl Iterator<Item = impl Borrow<Option<ChildReference<Self::HashOut>>>>,
        maybe_value: Option<&[u8]>,
    ) -> Vec<u8> {
        let mut nodes = Vec<Option<&[u8]>>::new();
        for child_ref in children {
            match child_ref.borrow() {
                Some(c) => match c {
                    ChildReference::Hash(h) => {
                        nodes.append(Some(&h))
                    },
                    ChildReference::Inline(inline_data, length) => {
                        nodes.append(Some(&inline_data))
                    },
                },
                None => nodes.append(None),
            };
        }
        let full = FullNode(&nodes, maybe_value);
        let encoded: Vec<u8> = bincode::serialize(&full).unwrap();
        encoded

    }

    fn branch_node_nibbled(
        _partial:   impl Iterator<Item = u8>,
        _number_nibble: usize,
        _children: impl Iterator<Item = impl Borrow<Option<ChildReference<Self::HashOut>>>>,
        _maybe_value: Option<&[u8]>) -> Vec<u8> {
        unreachable!("This codec is only used with a trie Layout that uses extension node.")
    }
}

// pub type TrieDBMut<'db> = trie_db::TrieDBMut<'db, KeccakHasher, RlpCodec>;

#[cfg(test)]
mod tests {
    // use hash_db::Hasher;
    // use reference_trie::{RefTrieDBMut, TrieMut};
    // use trie_db::DBValue;
    // use keccak_hasher::KeccakHasher;
    // use memory_db::*;

    // #[test]
    // fn test_triedb() {
    //     let mut memdb = MemoryDB::<KeccakHasher, HashKey<_>, DBValue>::default();
    //     let mut root = Default::default();
    //     let mut t = RefTrieDBMut::new(&mut memdb, &mut root);
    //     assert!(t.is_empty());
    //     assert_eq!(*t.root(), KeccakHasher::hash(&[0u8][..]));
    //     t.insert(b"foo", b"bar").unwrap();
    //     assert!(t.contains(b"foo").unwrap());
    //     assert_eq!(t.get(b"foo").unwrap().unwrap(), b"bar".to_vec());
    //     t.remove(b"foo").unwrap();
    //     assert!(!t.contains(b"foo").unwrap());
    // }
}
