//! Helpers to print a debug representations.

use crate::chunk::{Chunk, OpCode};

/// Given a chunk, prints its disassembly to `stdout`
pub fn disassemble_chunk(c: &Chunk, name: &str) {
    println!("== {name} ==");

    let mut offset = 0;
    while offset < c.len() {
        offset = disassemble_instruction(c, offset);
    }
}

/// Print one instruction from the [Chunk] to `stdout`, taking into account its operands.
pub fn disassemble_instruction(c: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0 && at_same_line_as_previous_offset(c, offset) {
        print!("   | ");
    } else {
        let line_no = c.line_number_for(offset).unwrap();
        print!("{line_no:4} ")
    }

    let instruction = c
        .get(offset)
        .expect("offset too large")
        .as_opcode()
        .expect("Invalid byte for opcode");

    use OpCode::*;
    #[allow(unreachable_patterns)]
    match instruction {
        // This is kind of silly in Rust, tbh
        Constant => constant_instruction("OP_CONSTANT", c, offset),
        Add => simple_instruction("OP_ADD", offset),
        Subtract => simple_instruction("OP_SUBTRACT", offset),
        Multiply => simple_instruction("OP_MULTIPLY", offset),
        Divide => simple_instruction("OP_DIVIDE", offset),
        Negate => simple_instruction("OP_NEGATE", offset),
        Return => simple_instruction("OP_RETURN", offset),
        BranchIfFalsy => branch_instruction("OP_BR_FALSE", c, offset),
        Jump => branch_instruction("OP_JUMP", c, offset),
    }
}

/////////////////////////////////////// Instruction printers ///////////////////////////////////////

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name:>14}");
    offset + 1
}

fn branch_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    print!("{name:>14}");
    let value = chunk
        .get(offset + 1)
        .expect("ran out of bytes")
        .as_constant_index();
    println!("  ->{value:04}");

    offset + 2
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    print!("{name:>14}");

    let (index, value) = chunk
        .get(offset + 1)
        .expect("ran out of bytes")
        .resolve_constant_with_index()
        .expect("Invalid constant index");

    println!("{index:4} '{value:?}'");

    offset + 2
}

//////////////////////////////////////////// Utilities ////////////////////////////////////////////

/// Returns true if the given offset is at the same line number as the previous line number.
fn at_same_line_as_previous_offset(chunk: &Chunk, offset: usize) -> bool {
    assert!(offset > 0);

    chunk
        .line_number_for(offset)
        .zip(chunk.line_number_for(offset - 1))
        .map(|(current_line, previous_line)| current_line == previous_line)
        .unwrap()
}
