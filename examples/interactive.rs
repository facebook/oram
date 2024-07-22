// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple interactive demonstration of ORAM.

use oram::path_oram::recursive_secure_path_oram::ConcreteObliviousBlockOram;
use oram::{BlockSize, Oram};
use rand::rngs::OsRng;
use rustyline::history::FileHistory;
use rustyline::Editor;

// The number of positions stored in each block used by the ORAM's recursive position map.
const ADDRESS_BLOCK_SIZE: BlockSize = 64;
type OramValue = u64;

fn parse_u64(
    prompt: &str,
    rl: &mut Editor<(), FileHistory>,
) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(loop {
        println!("{}", prompt);
        println!();
        let readline: String = rl.readline("> ")?;
        let number_parse = readline.parse::<u64>();
        match number_parse {
            Ok(number) => break number,
            Err(_) => {
                println!("Expected a u64. Try again.");
                continue;
            }
        }
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = OsRng;

    let mut rl = Editor::<(), _>::new().unwrap();

    println!("In this example, we initialize and interact with an oblivious RAM storing u64s.");

    let capacity = parse_u64("How many u64s would you like the ORAM to store?", &mut rl)?;

    // Initialize a Path ORAM storing `capacity` u64s.
    let mut oram =
        ConcreteObliviousBlockOram::<ADDRESS_BLOCK_SIZE, OramValue>::new(capacity, &mut rng)?;

    loop {
        let action = loop {
            println!("Enter an option (r or w):");
            println!("r) Read");
            println!("w) Write");
            let action: String = rl.readline("> ")?;
            if (action != "r") & (action != "w") {
                println!("Try again.");
                continue;
            }
            break action;
        };

        let address = parse_u64("What address?", &mut rl)?;

        if action == "r" {
            println!("Value at {} is {}.", address, oram.read(address, &mut rng)?);
        }

        if action == "w" {
            let value = parse_u64("Value to write?", &mut rl)?;
            oram.write(address, value, &mut rng)?;
            println!("Wrote value {} to address {}.", value, address);
        }
    }
}
