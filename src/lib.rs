// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of Oblivious RAM

type BlockSizeType = usize;
type IndexType = usize;

const BLOCK_SIZE: BlockSizeType = 4096;

pub trait ORAM {
    fn new(block_capacity: IndexType) -> Self;
    fn block_capacity(&self) -> IndexType;

    // Spencer: Ideally, this would be private, and only the derived methods read and write would be public.
    // However, the methods of a Rust trait are either all public or all private.
    // There seems to be a workaround involving modules, but for simplicity I do not do that here.
    fn access(&mut self, index: IndexType, optional_new_value: Option<BlockValue>) -> BlockValue;

    fn read(&mut self, index: IndexType) -> BlockValue {
        self.access(index, None)
    }

    fn write(&mut self, index: IndexType, new_value: BlockValue) {
        self.access(index, Some(new_value));
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BlockValue([u8; BLOCK_SIZE]);

impl BlockValue {
    fn encrypt(&self) -> BlockValue {
        self.clone()
    }
    fn decrypt(&self) -> BlockValue {
        self.clone()
    }
}

impl Default for BlockValue {
    fn default() -> Self {
        return BlockValue([0; BLOCK_SIZE]);
    }
}

struct LinearTimeORAM {
    memory: Vec<BlockValue>,
}

impl ORAM for LinearTimeORAM {
    fn new(block_capacity: IndexType) -> Self {
        return Self {
            memory: vec![BlockValue::default(); block_capacity],
        };
    }

    fn access(&mut self, index: IndexType, optional_new_value: Option<BlockValue>) -> BlockValue {
        assert!(index < self.block_capacity());

        let mut value_found_at_index = None;
        for i in 0..self.memory.len() {
            // Fetch entry
            let entry = self.memory[i];

            // Client-side processing
            let mut decrypted_entry = entry.decrypt();

            if i == index {
                value_found_at_index = Some(decrypted_entry);

                match optional_new_value {
                    None => {}
                    Some(value) => {
                        decrypted_entry = value;
                    }
                }
            }
            let recrypted_entry: BlockValue = decrypted_entry.encrypt();
            // End client-side processing

            // Write back entry
            self.memory[i] = recrypted_entry;
        }

        // value_found_at_index will always be a Some because of the bounds check on the input index
        value_found_at_index.unwrap()
    }

    fn block_capacity(&self) -> IndexType {
        self.memory.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_read_write() {
        let mut oram = LinearTimeORAM::new(16);
        let written_value = BlockValue([1; BLOCK_SIZE]);
        oram.write(0, written_value);
        let read_value = oram.read(0);
        assert_eq!(written_value, read_value);
    }
}
