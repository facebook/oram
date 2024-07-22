// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Implements a trait `Stash` representing a Path ORAM stash data structure.

use super::{bucket::Bucket, TreeIndex};
use crate::{database::Database, Address, BucketSize, OramBlock, OramError};

/// Numeric type used to represent the size of a Path ORAM stash in blocks.
pub type StashSize = u64;

/// A generic Path ORAM stash data structure.
pub trait Stash<V: OramBlock>
where
    Self: Sized,
{
    /// Creates a new stash capable of holding `capacity` blocks.
    fn new(path_size: StashSize, overflow_size: StashSize) -> Result<Self, OramError>;

    /// Reads blocks from the path specified by the leaf `position` in the tree `physical_memory` of height `height`.
    fn read_from_path<const Z: BucketSize, T: Database<Bucket<V, Z>>>(
        &mut self,
        physical_memory: &mut T,
        position: TreeIndex,
    ) -> Result<(), OramError>;

    /// Evicts blocks from the stash to the path specified by the leaf `position` in the tree `physical_memory` of height `height`.
    fn write_to_path<const Z: BucketSize, T: Database<Bucket<V, Z>>>(
        &mut self,
        physical_memory: &mut T,
        position: TreeIndex,
    ) -> Result<(), OramError>;

    /// Obliviously scans the stash for a block `b` with address `address`; if found, replaces that block with `callback(b)` and returns `b`.
    fn access<F: Fn(&V) -> V>(
        &mut self,
        address: Address,
        new_position: TreeIndex,
        value_callback: F,
    ) -> Result<V, OramError>;

    #[cfg(test)]
    /// The number of real (non-dummy) blocks in the stash.
    fn occupancy(&self) -> StashSize;
}
