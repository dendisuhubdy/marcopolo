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

extern crate starling;
extern crate serde;
extern crate map_store;

use std::error::Error;
use std::path::PathBuf;
use starling::traits::{Array, Database, Decode, Encode, Exception};
use starling::tree::tree_node::TreeNode;
use std::marker::PhantomData;
use map_store::mapdb;

pub struct MError(map_store::Error);
pub struct MWriteBatch(map_store::WriteBatch);
pub mod mapTree;

impl From<MError> for Exception {
    #[inline]
    fn from(error: MError) -> Self {
        Self::new(error.0.description())
    }
}

pub struct TreeDB<ArrayType>
where
    ArrayType: Array,
{
    // db: DB,
    // pending_inserts: Option<WriteBatch>,
    array: PhantomData<ArrayType>,
}

impl<ArrayType> TreeDB<ArrayType>
where
    ArrayType: Array,
{
    #[inline]
    // pub fn new(db: DB) -> Self {
    //     Self {
    //         db,
    //         pending_inserts: Some(WriteBatch::default()),
    //         array: PhantomData,
    //     }
    // }
    pub fn new() -> Self {
        Self {
            array: PhantomData,
        }
    }
}

impl<ArrayType> Database<ArrayType> for TreeDB<ArrayType>
where
    ArrayType: Array,
    TreeNode<ArrayType>: Encode + Decode,
{
    type NodeType = TreeNode<ArrayType>;
    type EntryType = (usize, usize);

    #[inline]
    fn open(path: &PathBuf) -> Result<Self, Exception> {
        // Ok(Self::new(DB::open_default(path)?))
        Ok(Self::new())
    }

    #[inline]
    fn get_node(&self, key: ArrayType) -> Result<Option<Self::NodeType>, Exception> {
        // if let Some(buffer) = self.db.get(&key)? {
        //     Ok(Some(Self::NodeType::decode(buffer.as_ref())?))
        // } else {
        //     Ok(None)
        // }
        Ok(None)
    }

    #[inline]
    fn insert(&mut self, key: ArrayType, value: Self::NodeType) -> Result<(), Exception> {
        // let serialized = value.encode()?;
        // if let Some(wb) = &mut self.pending_inserts {
        //     wb.put(key, serialized)?;
        // } else {
        //     let mut wb = WriteBatch::default();
        //     wb.put(key, serialized)?;
        //     self.pending_inserts = Some(wb);
        // }
        Ok(())
    }

    #[inline]
    fn remove(&mut self, key: &ArrayType) -> Result<(), Exception> {
        // Ok(self.db.delete(key)?)
        Ok(())
    }

    #[inline]
    fn batch_write(&mut self) -> Result<(), Exception> {
        // if let Some(wb) = self.pending_inserts.replace(WriteBatch::default()) {
        //     self.db.write(wb)?;
        // }
        // self.pending_inserts = None;
        Ok(())
    }
}
