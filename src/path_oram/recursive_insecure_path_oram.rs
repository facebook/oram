// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple, insecure implementation of Path ORAM with "client-side" stash and recursive position map.

use super::{
    address_oram_block::AddressOramBlock,
    generic_path_oram::GenericPathOram,
    generic_recursive_path_oram::{AddressOram, BlockOram},
    insecure_stash::VecStash,
    DEFAULT_BLOCKS_PER_BUCKET,
};
use crate::BlockSize;

/// An `Oram` intended for use as a position map, with a naive "client-side" stash. `AB` is the block size in addresses.
pub type ConcreteAddressOram<const AB: BlockSize, V> = GenericPathOram<
    V,
    DEFAULT_BLOCKS_PER_BUCKET,
    AddressOram<AB, DEFAULT_BLOCKS_PER_BUCKET, VecStash<AddressOramBlock<AB>>>,
    VecStash<V>,
>;

/// (!) This is a development stepping stone, not a finished implementation. (!)
/// A simple, insecure implementation of Path ORAM
/// with a recursive position map
/// whose stash is just a `Vec` of blocks that is accessed non-obliviously.
/// In our scenario where the stash is in untrusted storage,
/// this is of course completely unacceptable.
/// In the scenario where the stash is in trusted client storage,
/// the only leakage would be the size of the stash
/// and the positions of dummy blocks in the stash at the end of each access.
/// (Such leakage would likely still be unacceptable.)
/// The leakage will be addressed by more sophisticated stash access routines
/// in one of the next few iterations.
pub type ConcreteBlockOram<const B: BlockSize, V> =
    BlockOram<B, V, DEFAULT_BLOCKS_PER_BUCKET, VecStash<AddressOramBlock<B>>, VecStash<V>>;

#[cfg(test)]
mod address_oram_tests {
    use super::*;
    use crate::{block_value::*, path_oram::generic_path_oram::*, *};
    use crate::{test_utils::*, OramBlock};
    use core::iter::zip;

    create_correctness_tests_for_oram_type!(ConcreteAddressOram, AddressOramBlock);

    // Test that the stash size is not growing too large.
    type CAOStashSizeMonitor<const AB: BlockSize, V> = StashSizeMonitor<ConcreteAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(CAOStashSizeMonitor, AddressOramBlock);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type CAOConstantOccupancyMonitor<const AB: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(CAOConstantOccupancyMonitor, AddressOramBlock);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type CAOPhysicalAccessCountMonitor<const AB: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(CAOPhysicalAccessCountMonitor, AddressOramBlock);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct CAOAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteAddressOram<B, V>,
    }
    create_statistics_test_for_oram_type!(CAOAccessDistributionTester, BlockValue);
}

#[cfg(test)]
mod block_oram_tests {
    use crate::block_value::BlockValue;
    use crate::path_oram::*;
    use crate::test_utils::*;
    use crate::*;
    use core::iter::zip;
    use generic_path_oram::GenericPathOram;
    use recursive_insecure_path_oram::ConcreteBlockOram;

    create_correctness_tests_for_oram_type!(ConcreteBlockOram, BlockValue);

    // Test that the stash size is not growing too large.
    type CBOStashSizeMonitor<const AB: BlockSize, V> = StashSizeMonitor<ConcreteBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(CBOStashSizeMonitor, BlockValue);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type CBOConstantOccupancyMonitor<const AB: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(CBOConstantOccupancyMonitor, BlockValue);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type CBOCountPhysicalAccessesMonitor<const AB: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(CBOCountPhysicalAccessesMonitor, BlockValue);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct CBOAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteBlockOram<B, V>,
    }
    create_statistics_test_for_oram_type!(CBOAccessDistributionTester, BlockValue);
}
