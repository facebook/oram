// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A trait representing a Path ORAM position map.

use rand::{CryptoRng, RngCore};

use crate::{Address, BlockSize, Oram, ProtocolError};

use super::{address_oram_block::AddressOramBlock, TreeIndex};

pub trait PositionMap<const AB: BlockSize>: Oram<TreeIndex> {
    fn write_position_block<R: RngCore + CryptoRng>(
        &mut self,
        address: Address,
        position_block: AddressOramBlock<AB>,
        rng: &mut R,
    ) -> Result<(), ProtocolError>;
}
