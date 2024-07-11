// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of Oblivious RAM.

#![warn(clippy::cargo, clippy::doc_markdown, missing_docs, rustdoc::all)]

use rand::{CryptoRng, RngCore};
use subtle::ConditionallySelectable;

pub mod block_value;
pub mod database;
pub mod linear_time_oram;
pub mod path_oram;
pub mod utils;

#[cfg(test)]
mod test_utils;

/// The numeric type used to specify the size of an ORAM block in bytes.
pub type BlockSize = usize;
/// The numeric type used to specify the size of an ORAM in blocks, and to index into the ORAM.
pub type Address = usize;
/// The numeric type used to specify the size of an ORAM bucket in blocks.
pub type BucketSize = usize;

/// "Trait alias" for ORAM blocks: the values read and written by ORAMs.
pub trait OramBlock:
    Copy + Clone + std::fmt::Debug + Default + PartialEq + ConditionallySelectable
{
}

/// Represents an oblivious RAM (ORAM) mapping `OramAddress` addresses to `V: OramBlock` values.
/// `B` represents the size of each block of the ORAM in bytes.
pub trait Oram<V: OramBlock> {
    /// Returns a new ORAM mapping addresses `0 <= address < block_capacity` to default `V` values.
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, rng: &mut R) -> Self;

    /// Returns the capacity in blocks of this ORAM.
    fn block_capacity(&self) -> Address;

    /// Performs a (oblivious) ORAM access.
    /// Returns the value `v` previously stored at `index`, and writes `callback(v)` to `index`.
    fn access<R: RngCore + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: Address,
        callback: F,
        rng: &mut R,
    ) -> V;

    /// Obliviously reads the value stored at `index`.
    fn read<R: RngCore + CryptoRng>(&mut self, index: Address, rng: &mut R) -> V {
        log::debug!("ORAM read: {}", index);
        let callback = |x: &V| *x;
        self.access(index, callback, rng)
    }

    /// Obliviously writes the value stored at `index`. Returns the value previously stored at `index`.
    fn write<R: RngCore + CryptoRng>(&mut self, index: Address, new_value: V, rng: &mut R) -> V {
        log::debug!("ORAM write: {}", index);
        let callback = |_: &V| new_value;
        self.access(index, callback, rng)
    }
}
