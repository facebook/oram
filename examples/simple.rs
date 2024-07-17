// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A very simple demonstration of the use of ORAM.

extern crate oram;

use oram::path_oram::recursive_secure_path_oram::ConcreteObliviousBlockOram;
use oram::Oram;
use oram::ProtocolError;
use rand::rngs::OsRng;

fn main() -> Result<(), ProtocolError> {
    let mut rng = OsRng;
    let mut oram = ConcreteObliviousBlockOram::<64, u64>::new(64, &mut rng)?;
    oram.write(0, 1, &mut rng)?;
    println!("{}", oram.read(0, &mut rng)?);
    Ok(())
}
