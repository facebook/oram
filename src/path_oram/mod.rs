// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Implementations of Path ORAM.

use crate::BucketSize;
use path_oram_block::PathOramBlock;

pub(crate) type TreeIndex = u64;
type TreeHeight = u32;

/// The parameter "Z" from the Path ORAM literature that sets the number of blocks per bucket; typical values are 3 or 4.
/// Here we adopt the more conservative setting of 4.
pub const DEFAULT_BLOCKS_PER_BUCKET: BucketSize = 4;
const MAXIMUM_TREE_HEIGHT: TreeHeight = 63;

pub(crate) mod address_oram_block;
pub(crate) mod bitonic_sort;
pub(crate) mod bucket;
pub mod generic_path_oram;
pub mod generic_recursive_path_oram;
pub mod insecure_stash;
pub mod oblivious_stash;
mod path_oram_block;
pub mod recursive_insecure_path_oram;
pub mod recursive_secure_path_oram;
pub mod simple_insecure_path_oram;
pub mod simple_secure_path_oram;
pub mod stash;
mod tree_index;
