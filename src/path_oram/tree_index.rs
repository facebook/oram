// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Tree index

use super::{TreeHeight, TreeIndex};
use rand::{CryptoRng, Rng, RngCore};
use static_assertions::const_assert_eq;
use std::{mem::size_of, num::TryFromIntError};

const_assert_eq!(size_of::<TreeIndex>(), 8);

pub trait CompleteBinaryTreeIndex
where
    Self: Sized,
{
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self;
    fn random_leaf<R: RngCore + CryptoRng>(
        tree_height: TreeHeight,
        rng: &mut R,
    ) -> Result<Self, TryFromIntError>;
    fn ct_depth(&self) -> TreeHeight;
    fn is_leaf(&self, height: TreeHeight) -> bool;
    fn ct_common_ancestor_of_two_leaves(&self, other: Self) -> Self;
}

impl CompleteBinaryTreeIndex for TreeIndex {
    // A TreeIndex can have any nonzero value.
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self {
        // We maintain the invariant that all TreeIndex values are nonzero.
        assert_ne!(*self, 0);
        // We only call this method when the receiver is a leaf.
        assert!(self.is_leaf(height));

        let shift = height - depth;
        self >> shift
    }

    fn random_leaf<R: RngCore + CryptoRng>(
        tree_height: TreeHeight,
        rng: &mut R,
    ) -> Result<Self, TryFromIntError> {
        let tree_height: u32 = tree_height.try_into()?;
        let result = 2u64.pow(tree_height) + rng.gen_range(0..2u64.pow(tree_height));
        // The value we've just generated is at least the first summand, which is at least 1.
        assert_ne!(result, 0);
        Ok(result)
    }

    fn ct_depth(&self) -> TreeHeight {
        // We maintain the invariant that all TreeIndex values are nonzero.
        assert_ne!(*self, 0);

        let leading_zeroes: u64 = self.leading_zeros().into();
        let index_bitlength = 64;
        index_bitlength - leading_zeroes - 1
    }

    fn is_leaf(&self, height: TreeHeight) -> bool {
        // We maintain the invariant that all TreeIndex values are nonzero.
        assert_ne!(*self, 0);

        self.ct_depth() == height
    }

    fn ct_common_ancestor_of_two_leaves(&self, other: Self) -> Self {
        // We only call this function on pairs of Path ORAM leaves, which have the same depth.
        assert!(self.ct_depth() == other.ct_depth());

        let shared_prefix_length = (self ^ other).leading_zeros();
        let common_ancestor = self >> (Self::BITS - shared_prefix_length);

        // Since the input leaves are nonzero, the output must also be nonzero.
        assert_ne!(common_ancestor, 0);

        common_ancestor
    }
}
