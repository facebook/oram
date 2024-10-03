// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Utilities.

use crate::OramError;
use rand::seq::SliceRandom;
use rand::{CryptoRng, Rng, RngCore};

use subtle::{Choice, ConditionallySelectable, ConstantTimeGreater, ConstantTimeLess};

use std::num::TryFromIntError;

pub(crate) type TreeIndex = u64;
pub(crate) type TreeHeight = u64;

pub(crate) trait CompleteBinaryTreeIndex
where
    Self: Sized,
{
    fn ct_node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self;
    fn random_leaf<R: RngCore + CryptoRng>(
        tree_height: TreeHeight,
        rng: &mut R,
    ) -> Result<Self, TryFromIntError>;
    fn ct_depth(&self) -> TreeHeight;
    fn is_leaf(&self, height: TreeHeight) -> bool;
}

impl CompleteBinaryTreeIndex for TreeIndex {
    // A TreeIndex can have any nonzero value.
    fn ct_node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self {
        // We maintain the invariant that all TreeIndex values are nonzero.
        assert_ne!(*self, 0);
        // We only call this method when the receiver is a leaf.
        assert!(self.is_leaf(height));

        let shift = height - depth;
        self >> shift
    }

    fn random_leaf<R: RngCore + CryptoRng>(
        tree_height: TreeHeight,
        rng: &mut R,
    ) -> Result<Self, TryFromIntError> {
        let tree_height: u32 = tree_height.try_into()?;
        let result = 2u64.pow(tree_height) + rng.gen_range(0..2u64.pow(tree_height));
        // The value we've just generated is at least the first summand, which is at least 1.
        assert_ne!(result, 0);
        Ok(result)
    }

    fn ct_depth(&self) -> TreeHeight {
        // We maintain the invariant that all TreeIndex values are nonzero.
        assert_ne!(*self, 0);

        let leading_zeroes: u64 = self.leading_zeros().into();
        let index_bitlength = 64;
        index_bitlength - leading_zeroes - 1
    }

    fn is_leaf(&self, height: TreeHeight) -> bool {
        // We maintain the invariant that all TreeIndex values are nonzero.
        assert_ne!(*self, 0);

        self.ct_depth() == height
    }
}

/// Sorts `items` in ascending order of `keys`, obliviously and in constant time.
/// Assumes that `keys.len() == items.len()`.
/// The algorithm is bitonic sort, based on code written by Hans Werner Lang
/// and available [here](https://hwlang.de/algorithmen/sortieren/bitonic/oddn.htm).
pub(crate) fn bitonic_sort_by_keys<
    T: ConditionallySelectable,
    K: Ord + ConditionallySelectable + ConstantTimeGreater + ConstantTimeLess,
>(
    items: &mut [T],
    keys: &mut [K],
) {
    let ascending: Choice = 1.into();
    helper_bitonic_sort_by_keys(0, items.len(), items, keys, ascending);
}

fn helper_bitonic_sort_by_keys<
    T: ConditionallySelectable,
    K: Ord + ConditionallySelectable + ConstantTimeGreater + ConstantTimeLess,
>(
    lo: usize,
    n: usize,
    items: &mut [T],
    keys: &mut [K],
    direction: Choice,
) {
    if n > 1 {
        let m = n / 2;
        helper_bitonic_sort_by_keys(lo, m, items, keys, !direction);
        helper_bitonic_sort_by_keys(lo + m, n - m, items, keys, direction);
        helper_bitonic_merge_by_keys(lo, n, items, keys, direction);
    }
}

fn helper_bitonic_merge_by_keys<
    T: ConditionallySelectable,
    K: Ord + ConditionallySelectable + ConstantTimeGreater + ConstantTimeLess,
>(
    lo: usize,
    n: usize,
    items: &mut [T],
    keys: &mut [K],
    direction: Choice,
) {
    if n > 1 {
        let m = n.next_power_of_two() >> 1;
        for i in lo..(lo + n - m) {
            let j = i + m;
            let jlti = keys[j].ct_lt(&keys[i]);
            let do_swap = !(jlti ^ direction);
            let (items_i, items_j) = items.split_at_mut(i + 1);
            T::conditional_swap(&mut items_i[i], &mut items_j[j - (i + 1)], do_swap);
            let (keys_i, keys_j) = keys.split_at_mut(i + 1);
            K::conditional_swap(&mut keys_i[i], &mut keys_j[j - (i + 1)], do_swap);
        }

        helper_bitonic_merge_by_keys(lo, m, items, keys, direction);
        helper_bitonic_merge_by_keys(lo + m, n - m, items, keys, direction);
    }
}

/// Returns a random permutation of 0 through n.
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

/// Given a permutation, inverts it using oblivious (data-independent) operations.
pub(crate) fn invert_permutation_oblivious(permutation: &[u64]) -> Result<Vec<u64>, OramError> {
    let n: u64 = permutation.len().try_into()?;
    let mut copied = permutation.to_owned();
    let mut result = Vec::from_iter(0u64..n);
    bitonic_sort_by_keys(&mut result, &mut copied);
    Ok(result)
}

/// Converts a `Vec<u64>` to a `Vec<usize>`.
pub(crate) fn to_usize_vec(source: Vec<u64>) -> Result<Vec<usize>, OramError> {
    let mut result = Vec::new();
    for e in &source {
        let e: usize = (*e).try_into()?;
        result.push(e);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::TreeIndex;
    use rand::{rngs::StdRng, SeedableRng};
    use static_assertions::const_assert_eq;
    use std::mem::size_of;

    use super::{
        bitonic_sort_by_keys, invert_permutation_oblivious,
        random_permutation_of_0_through_n_exclusive,
    };

    #[test]
    fn check_size_of_tree_index() {
        const_assert_eq!(size_of::<TreeIndex>(), 8);
    }

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

    #[test]
    fn test_bitonic_sort() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut items: Vec<u64> = Vec::new();
        let mut keys: Vec<u64> = Vec::new();
        let n = 128;
        for e in random_permutation_of_0_through_n_exclusive(n, &mut rng) {
            items.push(e as u64);
            keys.push((e + (2 * n)) as u64);
        }

        bitonic_sort_by_keys(&mut items, &mut keys);
        for i in 0..(items.len() - 1) {
            assert!(keys[i] <= keys[i + 1]);
            assert_eq!(keys[i], items[i] + (2 * (n as u64)));
        }
    }
}
