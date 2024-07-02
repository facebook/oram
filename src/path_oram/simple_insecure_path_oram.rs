// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple, insecure implementation of Path ORAM with "client-side" stash and (non-recursive) position map.

use super::{generic_path_oram::GenericPathOram, insecure_stash::VecStash, TreeIndex};
use crate::{database::CountAccessesDatabase, BucketSize};

/// (!) This is a development stepping stone, not a finished implementation. (!)
/// A simple, insecure implementation of Path ORAM
/// with a non-recursive position map
/// whose stash is just a `Vec` of blocks that is accessed non-obliviously.
/// In our scenario where the stash is in untrusted storage,
/// this is of course completely unacceptable.
/// In the scenario where the stash is in trusted client storage,
/// the only leakage would be the size of the stash
/// and the positions of dummy blocks in the stash at the end of each access.
/// (Such leakage would likely still be unacceptable.)
/// The leakage will be addressed by more sophisticated stash access routines
/// in one of the next few iterations.
pub type SimpleInsecurePathOram<V, const Z: BucketSize> =
    GenericPathOram<V, Z, CountAccessesDatabase<TreeIndex>, VecStash<V>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block_value::*, path_oram::*, test_utils::*, *};
    use std::iter::zip;

    /// A type alias for a simple `SimpleInsecurePathOram` monomorphization.
    pub type ConcreteSimpleInsecurePathOram<const B: BlockSize, V> =
        SimpleInsecurePathOram<V, DEFAULT_BLOCKS_PER_BUCKET>;

    create_correctness_tests_for_oram_type!(ConcreteSimpleInsecurePathOram, BlockValue);

    // Test that the stash size is not growing too large.
    type SipoStashSizeMonitor<const B: BlockSize, V> =
        StashSizeMonitor<ConcreteSimpleInsecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoStashSizeMonitor, BlockValue);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type SipoConstantOccupancyMonitor<const B: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteSimpleInsecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoConstantOccupancyMonitor, BlockValue);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type SipoCountPhysicalAccessesMonitor<const B: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteSimpleInsecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoCountPhysicalAccessesMonitor, BlockValue);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct SipoAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteSimpleInsecurePathOram<B, V>,
    }
    create_statistics_test_for_oram_type!(SipoAccessDistributionTester, BlockValue);
}
