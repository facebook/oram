// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Tree index

use crate::path_oram::{TreeHeight, TreeIndex};
use rand::{CryptoRng, Rng, RngCore};
use std::mem::size_of;

pub trait CompleteBinaryTreeIndex {
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self;
    fn random_leaf<R: RngCore + CryptoRng>(tree_height: TreeHeight, rng: &mut R) -> Self;
    fn depth(&self) -> TreeHeight;
    fn is_leaf(&self, height: TreeHeight) -> bool;
}

impl CompleteBinaryTreeIndex for TreeIndex {
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self {
        assert_ne!(*self, 0);
        assert!(self.is_leaf(height));
        let shift = height - depth;
        self >> shift
    }

    fn random_leaf<R: RngCore + CryptoRng>(tree_height: TreeHeight, rng: &mut R) -> Self {
        2u64.pow(tree_height) + rng.gen_range(0..2u64.pow(tree_height))
    }

    fn depth(&self) -> TreeHeight {
        assert_ne!(*self, 0);
        let leading_zeroes = self.leading_zeros();
        let index_bitlength = 8 * (size_of::<TreeIndex>() as TreeHeight);
        index_bitlength - leading_zeroes - 1
    }

    fn is_leaf(&self, height: TreeHeight) -> bool {
        assert_ne!(*self, 0);
        self.depth() == height
    }
}
