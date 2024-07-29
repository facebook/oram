// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Block and bucket structures for Path ORAM.

use crate::{BlockSize, OramBlock};
use subtle::{Choice, ConditionallySelectable};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::BucketSize;

use crate::{utils::TreeIndex, Address};
use subtle::ConstantTimeEq;

#[derive(Clone, Copy, Debug, PartialEq)]
/// An `OramBlock` consisting of unstructured bytes.
pub struct BlockValue<const B: BlockSize>([u8; B]);

impl<const B: BlockSize> BlockValue<B> {
    /// Instantiates a `BlockValue` from an array of `BLOCK_SIZE` bytes.
    pub fn new(data: [u8; B]) -> Self {
        Self(data)
    }
}

impl<const B: BlockSize> Default for BlockValue<B> {
    fn default() -> Self {
        BlockValue::<B>([0u8; B])
    }
}

impl<const B: BlockSize> OramBlock for BlockValue<B> {}

impl<const B: BlockSize> ConditionallySelectable for BlockValue<B> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = BlockValue::default();
        for i in 0..B {
            result.0[i] = u8::conditional_select(&a.0[i], &b.0[i], choice);
        }
        result
    }
}

impl<const B: BlockSize> Distribution<BlockValue<B>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BlockValue<B> {
        let mut result = BlockValue::default();
        for i in 0..B {
            result.0[i] = rng.gen();
        }
        result
    }
}

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
pub struct PositionBlock<const B: BlockSize> {
    /// The Path ORAM positions stored in this block.
    pub data: [TreeIndex; B],
}

impl<const B: BlockSize> Default for PositionBlock<B> {
    fn default() -> Self {
        Self { data: [0; B] }
    }
}

impl<const B: BlockSize> ConditionallySelectable for PositionBlock<B> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = Self::default();
        for i in 0..B {
            result.data[i] = TreeIndex::conditional_select(&a.data[i], &b.data[i], choice);
        }
        result
    }
}

impl<const B: BlockSize> Distribution<PositionBlock<B>> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PositionBlock<B> {
        let mut result: PositionBlock<B> = PositionBlock::default();
        for i in 0..B {
            result.data[i] = rng.gen();
        }
        result
    }
}

impl<const B: BlockSize> OramBlock for PositionBlock<B> {}

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
