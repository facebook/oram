// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of a Path ORAM bucket.

use super::path_oram_block::PathOramBlock;
use crate::{BucketSize, OramBlock};
use subtle::{Choice, ConditionallySelectable};

#[repr(align(4096))]
#[derive(Clone, Copy, PartialEq)]
/// A Path ORAM bucket.
pub struct Bucket<V: OramBlock, const Z: BucketSize> {
    pub blocks: [PathOramBlock<V>; Z],
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
