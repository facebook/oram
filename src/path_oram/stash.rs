// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A trait representing a Path ORAM stash.

use super::{bucket::Bucket, TreeIndex};
use crate::{database::Database, Address, BucketSize, OramBlock};

/// Numeric type used to represent the size of a Path ORAM stash in blocks.
pub type StashSize = usize;

/// A Path ORAM stash.
pub trait Stash<V: OramBlock> {
    /// Creates a new stash capable of holding `capacity` blocks.
    fn new(path_size: StashSize, overflow_size: StashSize) -> Self;
    /// Read blocks from the path specified by the leaf `position` in the tree `physical_memory` of height `height`
    fn read_from_path<const Z: BucketSize, T: Database<Bucket<V, Z>>>(
        &mut self,
        physical_memory: &mut T,
        position: TreeIndex,
    );
    /// Evict blocks from the stash to the path specified by the leaf `position` in the tree `physical_memory` of height `height`.
    fn write_to_path<const Z: BucketSize, T: Database<Bucket<V, Z>>>(
        &mut self,
        physical_memory: &mut T,
        position: TreeIndex,
    );
    /// Obliviously scans the stash for a block `b` with address `address`; if found, replaces that block with `callback(b)`.
    fn access<F: Fn(&V) -> V>(
        &mut self,
        address: Address,
        new_position: TreeIndex,
        value_callback: F,
    ) -> V;

    #[cfg(test)]
    /// The number of real (non-dummy) blocks in the stash.
    fn occupancy(&self) -> StashSize;
}
