// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Implementations of Path ORAM.

use std::mem::size_of;

use crate::{Address, BucketSizeType, OramBlock, TreeHeight, TreeIndex};
use rand::Rng;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

pub mod simple_insecure_path_oram;

#[derive(Clone, Copy, Default, Debug)]
struct PathOramBlock<V> {
    value: V,
    address: Address,
    position: TreeIndex,
}

impl<V: OramBlock> PathOramBlock<V> {
    const DUMMY_ADDRESS: Address = Address::MAX;
    const DUMMY_POSITION: TreeIndex = u64::MAX;

    fn dummy() -> Self {
        Self {
            value: V::default(),
            address: Self::DUMMY_ADDRESS,
            position: Self::DUMMY_POSITION,
        }
    }

    fn ct_is_dummy(&self) -> Choice {
        self.address.ct_eq(&Self::DUMMY_ADDRESS)
    }
}

impl<V: ConditionallySelectable> ConditionallySelectable for PathOramBlock<V> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let value = V::conditional_select(&a.value, &b.value, choice);
        let address =
            u64::conditional_select(&(a.address as u64), &(b.address as u64), choice) as usize;
        let position = u64::conditional_select(&a.position, &b.position, choice);
        PathOramBlock::<V> {
            value,
            address,
            position,
        }
    }
}

#[repr(align(4096))]
#[derive(Clone, Copy, Debug)]
/// A Path ORAM bucket.
pub struct Bucket<V: OramBlock, const Z: BucketSizeType> {
    blocks: [PathOramBlock<V>; Z],
}

impl<V: OramBlock, const Z: BucketSizeType> Default for Bucket<V, Z> {
    fn default() -> Self {
        Self {
            blocks: [PathOramBlock::<V>::dummy(); Z],
        }
    }
}

trait CompleteBinaryTreeIndex {
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self;
    fn random_leaf<R: Rng>(tree_height: TreeHeight, rng: R) -> Self;
    fn depth(&self) -> TreeHeight;
    fn is_leaf(&self, height: TreeHeight) -> bool;
}

impl CompleteBinaryTreeIndex for TreeIndex {
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self {
        assert!(self.is_leaf(height));
        let shift = height - depth;
        self >> shift
    }

    fn random_leaf<R: Rng>(tree_height: TreeHeight, mut rng: R) -> Self {
        2u64.pow(tree_height) + rng.gen_range(0..2u64.pow(tree_height))
    }

    fn depth(&self) -> TreeHeight {
        let leading_zeroes = self.leading_zeros();
        let index_bitlength = 8 * (size_of::<TreeIndex>() as TreeHeight);
        index_bitlength - leading_zeroes - 1
    }

    fn is_leaf(&self, height: TreeHeight) -> bool {
        self.depth() == height
    }
}
