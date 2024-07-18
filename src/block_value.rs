// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Implements a generic ORAM value `BlockValue` consisting of unstructured bytes.

use crate::BlockSize;
use crate::OramBlock;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use subtle::{Choice, ConditionallySelectable};

impl OramBlock for u8 {}
impl OramBlock for u16 {}
impl OramBlock for u32 {}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(align(64))]
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
