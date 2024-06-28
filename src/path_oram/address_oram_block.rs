// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! This module contains common test utilities for crates generating tests utilizing the
//! `oram` crate.

use rand::distributions::{Distribution, Standard};
use subtle::{Choice, ConditionallySelectable};

use crate::{BlockSize, OramBlock};

use super::TreeIndex;

#[repr(align(64))]
#[derive(Clone, Copy, PartialEq, Debug)]

/// An `OramBlock` storing addresses, intended for use in a position map ORAM.
pub struct AddressOramBlock<const B: BlockSize> {
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
