// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of a Path ORAM with a recursive position map and obliviously accessed stash.

use super::{
    address_oram_block::AddressOramBlock,
    generic_path_oram::GenericPathOram,
    generic_recursive_path_oram::{AddressOram, BlockOram},
    oblivious_stash::BitonicStash,
    DEFAULT_BLOCKS_PER_BUCKET,
};
use crate::BlockSize;

/// A recursive ORAM intended for use as a position map, with an obliviously accessed stash.
pub type ConcreteObliviousAddressOram<const AB: BlockSize, V> = GenericPathOram<
    V,
    DEFAULT_BLOCKS_PER_BUCKET,
    AB,
    AddressOram<AB, DEFAULT_BLOCKS_PER_BUCKET, BitonicStash<AddressOramBlock<AB>>>,
    BitonicStash<V>,
>;

/// A secure Path ORAM with a recursive position map and obliviously accessed stash.
pub type ConcreteObliviousBlockOram<const B: BlockSize, V> =
    BlockOram<B, V, DEFAULT_BLOCKS_PER_BUCKET, BitonicStash<AddressOramBlock<B>>, BitonicStash<V>>;

#[cfg(test)]
mod address_oram_tests {
    use super::*;
    use crate::{block_value::*, path_oram::generic_path_oram::*, *};
    use crate::{test_utils::*, OramBlock};
    use core::iter::zip;

    create_correctness_tests_for_oram_type!(ConcreteObliviousAddressOram, AddressOramBlock);

    // Test that the stash size is not growing too large.
    type COAOStashSizeMonitor<const AB: BlockSize, V> =
        StashSizeMonitor<ConcreteObliviousAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COAOStashSizeMonitor, AddressOramBlock);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type COAOConstantOccupancyMonitor<const AB: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteObliviousAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COAOConstantOccupancyMonitor, AddressOramBlock);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type COAOPhysicalAccessCountMonitor<const AB: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteObliviousAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COAOPhysicalAccessCountMonitor, AddressOramBlock);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct COAOAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteObliviousAddressOram<B, V>,
    }
    create_statistics_test_for_oram_type!(COAOAccessDistributionTester, BlockValue);
}

#[cfg(test)]
mod block_oram_tests {
    use crate::block_value::BlockValue;
    use crate::path_oram::*;
    use crate::test_utils::*;
    use crate::*;
    use core::iter::zip;
    use generic_path_oram::GenericPathOram;
    use recursive_secure_path_oram::ConcreteObliviousBlockOram;

    create_correctness_tests_for_oram_type!(ConcreteObliviousBlockOram, BlockValue);

    // Test that the stash size is not growing too large.
    type COBOStashSizeMonitor<const AB: BlockSize, V> =
        StashSizeMonitor<ConcreteObliviousBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COBOStashSizeMonitor, BlockValue);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type COBOConstantOccupancyMonitor<const AB: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteObliviousBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COBOConstantOccupancyMonitor, BlockValue);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type COBOCountPhysicalAccessesMonitor<const AB: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteObliviousBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COBOCountPhysicalAccessesMonitor, BlockValue);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct COBOAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteObliviousBlockOram<B, V>,
    }
    create_statistics_test_for_oram_type!(COBOAccessDistributionTester, BlockValue);
}
