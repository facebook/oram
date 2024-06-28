// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A Path ORAM bucket.

use subtle::{Choice, ConditionallySelectable};

use crate::{BucketSize, OramBlock};

use super::path_oram_block::PathOramBlock;

#[repr(align(4096))]
#[derive(Clone, Copy, Debug, PartialEq)]
/// A Path ORAM bucket.
pub struct Bucket<V: OramBlock, const Z: BucketSize> {
    pub blocks: [PathOramBlock<V>; Z],
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
