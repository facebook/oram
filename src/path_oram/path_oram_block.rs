// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A Path ORAM block.

use crate::{path_oram::TreeIndex, Address, OramBlock};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

impl OramBlock for TreeIndex {}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct PathOramBlock<V> {
    pub value: V,
    pub address: Address,
    pub position: TreeIndex,
}

impl<V: OramBlock> PathOramBlock<V> {
    const DUMMY_ADDRESS: Address = Address::MAX;
    const DUMMY_POSITION: TreeIndex = 0;

    pub fn dummy() -> Self {
        Self {
            value: V::default(),
            address: Self::DUMMY_ADDRESS,
            position: Self::DUMMY_POSITION,
        }
    }

    pub fn ct_is_dummy(&self) -> Choice {
        self.position.ct_eq(&Self::DUMMY_POSITION)
    }

    pub fn is_dummy(&self) -> bool {
        self.position == Self::DUMMY_POSITION
    }
}

impl<V: OramBlock> std::fmt::Debug for PathOramBlock<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ct_is_dummy().into() {
            write!(f, "PathOramBlock::Dummy")
        } else {
            f.debug_struct("PathOramBlock")
                .field("value", &self.value)
                .field("address", &self.address)
                .field("position", &self.position)
                .finish()
        }
    }
}

impl<V: OramBlock> OramBlock for PathOramBlock<V> {}

impl<V: ConditionallySelectable> ConditionallySelectable for PathOramBlock<V> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let value = V::conditional_select(&a.value, &b.value, choice);
        let address = Address::conditional_select(&a.address, &b.address, choice);
        let position = TreeIndex::conditional_select(&a.position, &b.position, choice);
        PathOramBlock::<V> {
            value,
            address,
            position,
        }
    }
}
