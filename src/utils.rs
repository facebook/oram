// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Utilities.

use crate::path_oram::bitonic_sort::bitonic_sort_by_keys;
use crate::ProtocolError;
use rand::seq::SliceRandom;
use rand::{CryptoRng, RngCore};

pub(crate) fn random_permutation_of_0_through_n_exclusive<R: RngCore + CryptoRng>(
    n: u64,
    rng: &mut R,
) -> Vec<u64> {
    let permuted_addresses = 0..n;
    let mut permuted_addresses = Vec::from_iter(permuted_addresses);
    let permuted_addresses = permuted_addresses.as_mut_slice();
    permuted_addresses.shuffle(rng);
    Vec::from(permuted_addresses)
}

pub(crate) fn invert_permutation_oblivious(permutation: &[u64]) -> Result<Vec<u64>, ProtocolError> {
    let n: u64 = permutation.len().try_into()?;
    let mut copied = permutation.to_owned();
    let mut result = Vec::from_iter(0u64..n);
    bitonic_sort_by_keys(&mut result, &mut copied);
    Ok(result)
}

pub(crate) fn to_usize_vec(source: Vec<u64>) -> Result<Vec<usize>, ProtocolError> {
    let mut result = Vec::new();
    for e in &source {
        let e: usize = (*e).try_into()?;
        result.push(e);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng};

    use super::{invert_permutation_oblivious, random_permutation_of_0_through_n_exclusive};

    #[test]
    fn test_invert_permutation_oblivious() {
        let n = 16;
        let mut rng = StdRng::seed_from_u64(0);
        let permutation = random_permutation_of_0_through_n_exclusive(n, &mut rng);
        let inverse = invert_permutation_oblivious(&permutation).unwrap();
        for i in 0..n {
            assert_eq!(i, inverse[permutation[i as usize] as usize]);
        }
    }
}
