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

pub struct MapTree<ArrayType = [u8; 32], ValueType = Vec<u8>>
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

impl<ArrayType, ValueType> MapTree<ArrayType, ValueType>
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
    /// Get items from the `MapTree`.  Returns a map of `Option`s which may include the corresponding values.
    #[inline]
    pub fn get(
        &self,
        root_hash: &ArrayType,
        keys: &mut [ArrayType],
    ) -> BinaryMerkleTreeResult<HashMap<ArrayType, Option<ValueType>>> {
        self.tree.get(root_hash, keys)
    }

    /// Gets a single key from the tree.
    #[inline]
    pub fn get_one(
        &self,
        root: &ArrayType,
        key: &ArrayType,
    ) -> BinaryMerkleTreeResult<Option<ValueType>> {
        self.tree.get_one(&root, &key)
    }

    /// Insert items into the `MapTree`.  Keys must be sorted.  Returns a new root hash for the `MapTree`.
    #[inline]
    pub fn insert(
        &mut self,
        previous_root: Option<&ArrayType>,
        keys: &mut [ArrayType],
        values: &[ValueType],
    ) -> BinaryMerkleTreeResult<ArrayType> {
        self.tree.insert(previous_root, keys, values)
    }

    /// Inserts a single value into a tree.
    #[inline]
    pub fn insert_one(
        &mut self,
        previous_root: Option<&ArrayType>,
        key: &ArrayType,
        value: &ValueType,
    ) -> BinaryMerkleTreeResult<ArrayType> {
        self.tree.insert_one(previous_root, key, value)
    }

    /// Remove all items with less than 1 reference under the given root.
    #[inline]
    pub fn remove(&mut self, root_hash: &ArrayType) -> BinaryMerkleTreeResult<()> {
        self.tree.remove(root_hash)
    }

    /// Generates an inclusion proof.  The proof consists of a list of hashes beginning with the key/value
    /// pair and traveling up the tree until the level below the root is reached.
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


#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    use starling::constants::KEY_LEN;
    use starling::merkle_bit::BinaryMerkleTreeResult;
    use starling::traits::Exception;

    fn generate_path(seed: [u8; KEY_LEN]) -> PathBuf {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let suffix = rng.gen_range(1000, 100000);
        let path_string = format!("Test_DB_{}", suffix);
        PathBuf::from(path_string)
    }
    fn tear_down(_path: &PathBuf) {
        use std::fs::remove_dir_all;
        remove_dir_all(&_path).unwrap();
    }

    #[test]
    #[cfg(feature = "use_serialization")]
    fn test01_real_database() -> BinaryMerkleTreeResult<()> {
        let seed = [0x00u8; KEY_LEN];
        let path = generate_path(seed);
        let key = [0xAAu8; KEY_LEN];
        let retrieved_value;
        let removed_retrieved_value;
        let data = vec![0xFFu8];
        {
            let values = vec![data.clone()];
            let mut tree = MapTree::open(&path, 160)?;
            let root;
            match tree.insert(None, &mut [key], &values) {
                Ok(r) => root = r,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", e.description());
                }
            }
            match tree.get(&root, &mut [key]) {
                Ok(v) => retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", e.description());
                }
            }
            match tree.remove(&root) {
                Ok(_) => {}
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", e.description());
                }
            }
            match tree.get(&root, &mut [key]) {
                Ok(v) => removed_retrieved_value = v,
                Err(e) => {
                    drop(tree);
                    tear_down(&path);
                    panic!("{:?}", e.description());
                }
            }
        }
        tear_down(&path);
        assert_eq!(retrieved_value[&key], Some(data));
        assert_eq!(removed_retrieved_value[&key], None);
        Ok(())
    }
}
