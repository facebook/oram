// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple interactive demonstration of ORAM.

use oram::path_oram::recursive_secure_path_oram::ConcreteObliviousBlockOram;
use oram::Oram;
use rand::rngs::OsRng;
use rustyline::history::FileHistory;
use rustyline::Editor;

fn parse_number(
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
                println!("Expected a number. Try again.");
                continue;
            }
        }
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = OsRng;

    let mut rl = Editor::<(), _>::new().unwrap();

    let capacity = parse_number("How many integers would you like to store?", &mut rl)?;

    let mut oram = ConcreteObliviousBlockOram::<64, u64>::new(capacity, &mut rng)?;

    loop {
        let action = loop {
            println!("Enter an option (R or W):");
            println!("R) Read");
            println!("W) Write");
            let action: String = rl.readline("> ")?;
            if (action != "R") & (action != "W") {
                println!("Try again.");
                continue;
            }
            break action;
        };

        let address = parse_number("What address?", &mut rl)?;

        if action == "R" {
            println!("Value at {} is {}.", address, oram.read(address, &mut rng)?);
        }

        if action == "W" {
            let value = parse_number("Value to write?", &mut rl)?;
            oram.write(address, value, &mut rng)?;
            println!("Wrote value {} to address {}.", value, address);
        }
    }
}
