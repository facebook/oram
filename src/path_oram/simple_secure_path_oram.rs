// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A Path ORAM with a simple (non-recursive) position map and an obliviously accessed stash.

use super::{generic_path_oram::GenericPathOram, oblivious_stash::BitonicStash, TreeIndex};
use crate::{database::CountAccessesDatabase, BucketSize};

/// A Path ORAM with a simple (non-recursive) position map and an obliviously accessed stash.
pub type SimpleSecurePathOram<V, const Z: BucketSize> =
    GenericPathOram<V, Z, CountAccessesDatabase<TreeIndex>, BitonicStash<V>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block_value::*, path_oram::*, test_utils::*, *};
    use std::iter::zip;

    /// A convenient specialization of `SimpleSecurePathOram` for testing and benchmarking.
    pub type ConcreteSimpleSecurePathOram<const B: BlockSize, V> =
        SimpleSecurePathOram<V, DEFAULT_BLOCKS_PER_BUCKET>;

    create_correctness_tests_for_oram_type!(ConcreteSimpleSecurePathOram, BlockValue);

    // Test that the stash size is not growing too large.
    type SipoStashSizeMonitor<const B: BlockSize, V> =
        StashSizeMonitor<ConcreteSimpleSecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoStashSizeMonitor, BlockValue);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type SipoConstantOccupancyMonitor<const B: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteSimpleSecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoConstantOccupancyMonitor, BlockValue);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type SipoCountPhysicalAccessesMonitor<const B: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteSimpleSecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoCountPhysicalAccessesMonitor, BlockValue);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct SipoAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteSimpleSecurePathOram<B, V>,
    }
    create_statistics_test_for_oram_type!(SipoAccessDistributionTester, BlockValue);
}
