// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Tree index

use crate::InternalError;

use super::{TreeHeight, TreeIndex};
use rand::{CryptoRng, Rng, RngCore};
use static_assertions::const_assert_eq;
use std::{mem::size_of, num::TryFromIntError};

const_assert_eq!(size_of::<TreeIndex>(), 8);

pub trait CompleteBinaryTreeIndex
where
    Self: Sized,
{
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Result<Self, InternalError>;
    fn random_leaf<R: RngCore + CryptoRng>(
        tree_height: TreeHeight,
        rng: &mut R,
    ) -> Result<Self, TryFromIntError>;
    fn ct_depth(&self) -> Result<TreeHeight, InternalError>;
    fn is_leaf(&self, height: TreeHeight) -> Result<bool, InternalError>;
    fn ct_common_ancestor_of_two_leaves(&self, other: Self) -> Result<Self, InternalError>;
}

impl CompleteBinaryTreeIndex for TreeIndex {
    // A TreeIndex can have any nonzero value.
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Result<Self, InternalError> {
        if (*self == 0) | (!self.is_leaf(height)?) {
            return Err(InternalError::TreeIndexError { index: *self });
        }

        let shift = height - depth;
        Ok(self >> shift)
    }

    fn random_leaf<R: RngCore + CryptoRng>(
        tree_height: TreeHeight,
        rng: &mut R,
    ) -> Result<Self, TryFromIntError> {
        let tree_height: u32 = tree_height.try_into()?;
        Ok(2u64.pow(tree_height) + rng.gen_range(0..2u64.pow(tree_height)))
    }

    fn ct_depth(&self) -> Result<TreeHeight, InternalError> {
        if *self == 0 {
            return Err(InternalError::TreeIndexError { index: *self });
        }

        let leading_zeroes: u64 = self.leading_zeros().into();
        let index_bitlength = 64;
        Ok(index_bitlength - leading_zeroes - 1)
    }

    fn is_leaf(&self, height: TreeHeight) -> Result<bool, InternalError> {
        if *self == 0 {
            return Err(InternalError::TreeIndexError { index: *self });
        }
        Ok(self.ct_depth()? == height)
    }

    fn ct_common_ancestor_of_two_leaves(&self, other: Self) -> Result<Self, InternalError> {
        // The two inputs must be of the same height.
        if self.leading_zeros() != other.leading_zeros() {
            return Err(InternalError::TreeIndexingError);
        }

        let shared_prefix_length = (self ^ other).leading_zeros();
        let common_ancestor = self >> (Self::BITS - shared_prefix_length);

        if common_ancestor == 0 {
            return Err(InternalError::TreeIndexError {
                index: common_ancestor,
            });
        }
        Ok(common_ancestor)
    }
}
