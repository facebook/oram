// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Implements a trait `PositionMap` representing a Path ORAM position map data structure.

use rand::{CryptoRng, RngCore};

use crate::{utils::TreeIndex, Address, BlockSize, Oram, OramError};

use super::AddressOramBlock;

pub trait PositionMap<const AB: BlockSize>: Oram<TreeIndex> {
    fn write_position_block<R: RngCore + CryptoRng>(
        &mut self,
        address: Address,
        position_block: AddressOramBlock<AB>,
        rng: &mut R,
    ) -> Result<(), OramError>;
}
