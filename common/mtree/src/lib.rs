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
use map_store::{mapdb::MapDB,Config};

pub struct MError(map_store::Error);
pub struct MWriteBatch(map_store::WriteBatch);
impl Default for MWriteBatch {
    fn default() -> Self {
        MWriteBatch(map_store::WriteBatch::default())
    }
}

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
    db: MapDB,
    pending_inserts: Option<MWriteBatch>,
    array: PhantomData<ArrayType>,
}

impl<ArrayType> TreeDB<ArrayType>
where
    ArrayType: Array,
{
    #[inline]
    pub fn new(db: MapDB) -> Self {
        Self {
            db,
            pending_inserts: Some(MWriteBatch::default()),
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
        let mut p = path.clone();
        let cfg = Config::new(p);
        let res = MapDB::open(cfg);
        match res {
            Ok(db) => Ok(Self::new(db)),
            Err(e) => Err(MError(e).into()),
        }
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
