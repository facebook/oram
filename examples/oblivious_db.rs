// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An example of using ORAM to obliviously serve an indexed database.

extern crate oram;

use oram::{Address, BlockSize, BlockValue, DefaultOram, Oram, OramError};
use rand::{rngs::OsRng, Rng};

const BLOCK_SIZE: BlockSize = 4096;
const DB_SIZE: Address = 64;
// A stand-in for the indexed database you want to obliviously serve.
const DATABASE: [[u8; BLOCK_SIZE as usize]; DB_SIZE as usize] =
    [[0; BLOCK_SIZE as usize]; DB_SIZE as usize];

fn main() -> Result<(), OramError> {
    let mut rng = OsRng;
    let mut oram = DefaultOram::<BlockValue<4096>>::new(DB_SIZE, &mut rng)?;

    // Read DATABASE into oram.
    for (i, bytes) in DATABASE.iter().enumerate() {
        oram.write(i as Address, BlockValue::new(*bytes), &mut rng)?;
    }

    // Now oram can be used to obliviously serve the contents of DATABASE.
    let num_operations = 100;
    for _ in 0..num_operations {
        let random_index = rng.gen_range(0..DB_SIZE);

        let _ = oram.read(random_index, &mut rng)?;
    }

    Ok(())
}
