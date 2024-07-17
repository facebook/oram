// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of the Path ORAM oblivious RAM protocol.
//!
//! ### Minimum Supported Rust Version
//! 
//! Rust **1.67.1** or higher. (TODO check!)
//! 
//! # Overview
//! 
//! Oblivious RAM is a protocol between a client and a store. 
//! The client makes read(i) and write(i, data) requests via the ORAM protocol
//! just as if it were interacting with a random access memory (RAM).
//! The protocol makes read(i) and write(i, data) requests to the store
//! in order to service the client's requests,
//! while guaranteeing that the sequence of requests seen by the store
//! is independent of the sequence of requests made by the client
//! (up to the length of the client request sequence).

#![warn(clippy::cargo, clippy::doc_markdown, missing_docs, rustdoc::all)]

use std::num::TryFromIntError;

use path_oram::TreeIndex;
use rand::{CryptoRng, RngCore};
use subtle::ConditionallySelectable;
use thiserror::Error;

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
pub type Address = u64;
/// The numeric type used to specify the size of an ORAM bucket in blocks.
pub type BucketSize = usize;

/// "Trait alias" for ORAM blocks: the values read and written by ORAMs.
pub trait OramBlock:
    Copy + Clone + std::fmt::Debug + Default + PartialEq + ConditionallySelectable
{
}

/// Represents an error in the internal operations of the library
#[derive(Error, Debug)]
pub enum InternalError {
    /// Invalid tree index produced.
    #[error("Invalid tree index {index} produced.")]
    TreeIndexError {
        /// The invalid tree index.
        index: TreeIndex,
    },
    /// Invalid tree indexing operation.
    #[error("Invalid tree indexing operation.")]
    TreeIndexingError,
}

/// A list of error types which are produced during ORAM protocol execution
#[derive(Error, Debug)]
pub enum ProtocolError {
    // TODO: this should be an internal error. Ask Kevin how to handle chaining of errors.
    /// Errors arising from conversions between integer types.
    #[error("Arithmetic error encountered.")]
    IntegerConversionError(#[from] TryFromIntError),
    /// Internal library error
    #[error("Internal library error.")]
    LibraryError(#[from] InternalError),
    /// ORAM access to invalid address
    #[error("Attempted to access an out-of-bounds ORAM address.")]
    AddressOutOfBoundsError,
    /// Errors having to do with parameters.
    #[error("Invalid configuration.")]
    InvalidConfigurationError,
}

/// Represents an oblivious RAM (ORAM) mapping `OramAddress` addresses to `V: OramBlock` values.
/// `B` represents the size of each block of the ORAM in bytes.
pub trait Oram<V: OramBlock>
where
    Self: Sized,
{
    /// Returns a new ORAM mapping addresses `0 <= address < block_capacity` to default `V` values.
    fn new<R: RngCore + CryptoRng>(
        block_capacity: Address,
        rng: &mut R,
    ) -> Result<Self, ProtocolError>;

    /// Returns the capacity in blocks of this ORAM.
    fn block_capacity(&self) -> Result<Address, ProtocolError>;

    /// Performs a (oblivious) ORAM access.
    /// Returns the value `v` previously stored at `index`, and writes `callback(v)` to `index`.
    fn access<R: RngCore + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: Address,
        callback: F,
        rng: &mut R,
    ) -> Result<V, ProtocolError>;

    /// Obliviously reads the value stored at `index`.
    fn read<R: RngCore + CryptoRng>(
        &mut self,
        index: Address,
        rng: &mut R,
    ) -> Result<V, ProtocolError> {
        log::debug!("ORAM read: {}", index);
        let callback = |x: &V| *x;
        self.access(index, callback, rng)
    }

    /// Obliviously writes the value stored at `index`. Returns the value previously stored at `index`.
    fn write<R: RngCore + CryptoRng>(
        &mut self,
        index: Address,
        new_value: V,
        rng: &mut R,
    ) -> Result<V, ProtocolError> {
        log::debug!("ORAM write: {}", index);
        let callback = |_: &V| new_value;
        self.access(index, callback, rng)
    }
}
