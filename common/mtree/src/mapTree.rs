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

#[cfg(not(any(feature = "use_hashbrown")))]
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "use_hashbrown")]
use hashbrown::HashMap;

use starling::merkle_bit::{BinaryMerkleTreeResult, MerkleBIT};
use starling::traits::{Array, Database, Decode, Encode};
use starling::tree::tree_branch::TreeBranch;
use starling::tree::tree_data::TreeData;
use starling::tree::tree_leaf::TreeLeaf;
use starling::tree::tree_node::TreeNode;
use starling::tree_hasher::TreeHasher;
// #[cfg(feature = "use_serde")]
use serde::de::DeserializeOwned;
// #[cfg(feature = "use_serde")]
use serde::Serialize;
use super::TreeDB;

pub struct mapTree<ArrayType = [u8; 32], ValueType = Vec<u8>>
where
    ArrayType: Array + Serialize + DeserializeOwned,
    ValueType: Encode + Decode,
{
    tree: MerkleBIT<
        TreeDB<ArrayType>,
        TreeBranch<ArrayType>,
        TreeLeaf<ArrayType>,
        TreeData,
        TreeNode<ArrayType>,
        TreeHasher,
        ValueType,
        ArrayType,
    >,
}

impl<ArrayType, ValueType> mapTree<ArrayType, ValueType>
where
    ArrayType: Array + Serialize + DeserializeOwned,
    ValueType: Encode + Decode,
{
    #[inline]
    pub fn open(path: &PathBuf, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let db = TreeDB::open(path)?;
        let tree = MerkleBIT::from_db(db, depth)?;
        Ok(Self { tree })
    }

    #[inline]
    pub fn from_db(db: TreeDB<ArrayType>, depth: usize) -> BinaryMerkleTreeResult<Self> {
        let tree = MerkleBIT::from_db(db, depth)?;
        Ok(Self { tree })
    }

    #[inline]
    pub fn get(
        &self,
        root_hash: &ArrayType,
        keys: &mut [ArrayType],
    ) -> BinaryMerkleTreeResult<HashMap<ArrayType, Option<ValueType>>> {
        self.tree.get(root_hash, keys)
    }

    #[inline]
    pub fn get_one(
        &self,
        root: &ArrayType,
        key: &ArrayType,
    ) -> BinaryMerkleTreeResult<Option<ValueType>> {
        self.tree.get_one(&root, &key)
    }

    #[inline]
    pub fn insert(
        &mut self,
        previous_root: Option<&ArrayType>,
        keys: &mut [ArrayType],
        values: &[ValueType],
    ) -> BinaryMerkleTreeResult<ArrayType> {
        self.tree.insert(previous_root, keys, values)
    }

    #[inline]
    pub fn insert_one(
        &mut self,
        previous_root: Option<&ArrayType>,
        key: &ArrayType,
        value: &ValueType,
    ) -> BinaryMerkleTreeResult<ArrayType> {
        self.tree.insert_one(previous_root, key, value)
    }

    #[inline]
    pub fn remove(&mut self, root_hash: &ArrayType) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }

    #[inline]
    pub fn generate_inclusion_proof(
        &self,
        root: &ArrayType,
        key: ArrayType,
    ) -> BinaryMerkleTreeResult<Vec<(ArrayType, bool)>> {
        self.tree.generate_inclusion_proof(root, key)
    }

    #[inline]
    pub fn verify_inclusion_proof(
        &self,
        root: &ArrayType,
        key: ArrayType,
        value: &ValueType,
        proof: &Vec<(ArrayType, bool)>,
    ) -> BinaryMerkleTreeResult<()> {
        self.tree.verify_inclusion_proof(root, key, value, proof)
    }
}
