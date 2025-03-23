use crate::emulator::cpu::{Instruction, INSTRUCTIONS};

pub fn decode_raw<const XLEN: u8>(word: u32) -> Result<Instruction<XLEN>, ()> {
    match decode_and_get_instruction_index::<XLEN>(word) {
        Ok(index) => Ok(INSTRUCTIONS[index].clone()),
        Err(()) => Err(()),
    }
}

fn decode_and_get_instruction_index<const XLEN: u8>(word: u32) -> Result<usize, ()> {
    for (i, inst) in INSTRUCTIONS::<{XLEN}>.iter().enumerate() {
        if (word & inst.mask) == inst.data {
            return Ok(i);
        }
    }
    Err(())
}
