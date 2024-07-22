// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Implementations of Path ORAM.

use crate::BucketSize;

/// The parameter "Z" from the Path ORAM literature that sets the number of blocks per bucket; typical values are 3 or 4.
/// Here we adopt the more conservative setting of 4.
pub const DEFAULT_BLOCKS_PER_BUCKET: BucketSize = 4;

pub use stash::Stash;

pub(crate) mod generic_path_oram;
pub(crate) mod generic_recursive_path_oram;
pub(crate) mod position_map;
pub mod recursive_secure_path_oram;
pub(crate) mod stash;

use crate::{utils::TreeIndex, Address, BlockSize, OramBlock};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

use rand::distributions::{Distribution, Standard};

#[derive(Clone, Copy, Default, PartialEq)]
/// A Path ORAM block combines an `OramBlock` V with two metadata fields; its ORAM `address` and its `position` in the tree.
pub(crate) struct PathOramBlock<V> {
    pub value: V,
    pub address: Address,
    pub position: TreeIndex,
}

impl<V: OramBlock> PathOramBlock<V> {
    const DUMMY_ADDRESS: Address = Address::MAX;
    const DUMMY_POSITION: TreeIndex = 0;

    pub fn dummy() -> Self {
        Self {
            value: V::default(),
            address: Self::DUMMY_ADDRESS,
            position: Self::DUMMY_POSITION,
        }
    }

    pub fn ct_is_dummy(&self) -> Choice {
        self.position.ct_eq(&Self::DUMMY_POSITION)
    }

    #[cfg(test)]
    pub fn is_dummy(&self) -> bool {
        self.position == Self::DUMMY_POSITION
    }
}

impl<V: OramBlock> std::fmt::Debug for PathOramBlock<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ct_is_dummy().into() {
            write!(f, "PathOramBlock::Dummy")
        } else {
            f.debug_struct("PathOramBlock")
                .field("value", &self.value)
                .field("address", &self.address)
                .field("position", &self.position)
                .finish()
        }
    }
}

impl<V: OramBlock> OramBlock for PathOramBlock<V> {}

impl<V: ConditionallySelectable> ConditionallySelectable for PathOramBlock<V> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let value = V::conditional_select(&a.value, &b.value, choice);
        let address = Address::conditional_select(&a.address, &b.address, choice);
        let position = TreeIndex::conditional_select(&a.position, &b.position, choice);
        PathOramBlock::<V> {
            value,
            address,
            position,
        }
    }
}

#[repr(align(64))]
#[derive(Clone, Copy, PartialEq, Debug)]
/// An `OramBlock` storing addresses, intended for use in a position map ORAM.
pub struct AddressOramBlock<const B: BlockSize> {
    /// The Path ORAM positions stored in this block.
    pub data: [TreeIndex; B],
}

impl<const B: BlockSize> Default for AddressOramBlock<B> {
    fn default() -> Self {
        Self { data: [0; B] }
    }
}

impl<const B: BlockSize> ConditionallySelectable for AddressOramBlock<B> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = Self::default();
        for i in 0..B {
            result.data[i] = TreeIndex::conditional_select(&a.data[i], &b.data[i], choice);
        }
        result
    }
}

impl<const B: BlockSize> Distribution<AddressOramBlock<B>> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> AddressOramBlock<B> {
        let mut result: AddressOramBlock<B> = AddressOramBlock::default();
        for i in 0..B {
            result.data[i] = rng.gen();
        }
        result
    }
}

impl<const B: BlockSize> OramBlock for AddressOramBlock<B> {}

#[repr(align(4096))]
#[derive(Clone, Copy, PartialEq)]
/// A Path ORAM bucket.
pub struct Bucket<V: OramBlock, const Z: BucketSize> {
    /// The Path ORAM blocks stored by this bucket.
    pub(crate) blocks: [PathOramBlock<V>; Z],
}

impl<V: OramBlock, const Z: BucketSize> std::fmt::Debug for Bucket<V, Z> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut self_is_dummy = true;

        for block in self.blocks {
            if (!block.ct_is_dummy()).into() {
                self_is_dummy = false;
            }
        }

        if self_is_dummy {
            write!(f, "Bucket::Dummy")
        } else {
            f.debug_struct("Bucket")
                .field("blocks", &self.blocks)
                .finish()
        }
    }
}

impl<V: OramBlock, const Z: BucketSize> Default for Bucket<V, Z> {
    fn default() -> Self {
        Self {
            blocks: [PathOramBlock::<V>::dummy(); Z],
        }
    }
}

impl<V: OramBlock, const Z: BucketSize> ConditionallySelectable for Bucket<V, Z> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = Self::default();
        for i in 0..result.blocks.len() {
            result.blocks[i] =
                PathOramBlock::<V>::conditional_select(&a.blocks[i], &b.blocks[i], choice)
        }
        result
    }
}

impl<V: OramBlock, const Z: BucketSize> OramBlock for Bucket<V, Z> {}
