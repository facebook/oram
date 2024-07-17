// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of bitonic sort.

use subtle::{
    Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess,
};

/// Sorts `items` in ascending order of `keys`, obliviously and in constant time.
/// Assumes that `items.len()` is a power of two and that `keys.len() == items.len()`.
/// The algorithm is a non-recursive version of bitonic sort based on
/// pseudocode from [Wikipedia](https://en.wikipedia.org/wiki/Bitonic_sorter#Example_code).
pub(crate) fn bitonic_sort_by_keys<
    T: ConditionallySelectable,
    K: Ord + ConditionallySelectable + ConstantTimeGreater + ConstantTimeLess,
>(
    items: &mut [T],
    keys: &mut [K],
) {
    let n = items.len();
    assert!(n.is_power_of_two()); // This is already checked in oblivious stash initialization.

    let mut k = 2;
    while k <= n {
        let mut j = k / 2;
        while j > 0 {
            for i in 0..n {
                let l = i ^ j;
                if l > i {
                    let ik0: Choice = (i & k).ct_eq(&0);
                    let igtl: Choice = keys[i].ct_gt(&keys[l]);
                    let iltl: Choice = keys[i].ct_lt(&keys[l]);
                    let do_swap = (ik0 & igtl) | ((!ik0) & iltl);
                    let (items_i, items_l) = items.split_at_mut(i + 1);
                    T::conditional_swap(&mut items_i[i], &mut items_l[l - (i + 1)], do_swap);
                    let (keys_i, keys_l) = keys.split_at_mut(i + 1);
                    K::conditional_swap(&mut keys_i[i], &mut keys_l[l - (i + 1)], do_swap);
                }
            }
            j /= 2;
        }
        k *= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::random_permutation_of_0_through_n_exclusive;
    use rand::{rngs::StdRng, SeedableRng};

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
