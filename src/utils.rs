// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Utilities.

use crate::Address;
use rand::seq::SliceRandom;
use rand::{CryptoRng, RngCore};

pub(crate) fn random_permutation_of_0_through_n_exclusive<R: RngCore + CryptoRng>(
    n: Address,
    rng: &mut R,
) -> Vec<Address> {
    let permuted_addresses = 0..n;
    let mut permuted_addresses = Vec::from_iter(permuted_addresses);
    let permuted_addresses = permuted_addresses.as_mut_slice();
    permuted_addresses.shuffle(rng);
    Vec::from(permuted_addresses)
}
