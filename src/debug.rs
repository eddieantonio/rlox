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
        Nil => simple_instruction("OP_NIL", offset),
        True => simple_instruction("OP_TRUE", offset),
        False => simple_instruction("OP_FALSE", offset),
        Pop => simple_instruction("OP_POP", offset),
        GetGlobal => constant_instruction("OP_GET_GLOBAL", c, offset),
        DefineGlobal => constant_instruction("OP_DEFINE_GLOBAL", c, offset),
        Equal => simple_instruction("OP_EQUAL", offset),
        Greater => simple_instruction("OP_GREATER", offset),
        Less => simple_instruction("OP_LESS", offset),
        Add => simple_instruction("OP_ADD", offset),
        Subtract => simple_instruction("OP_SUBTRACT", offset),
        Multiply => simple_instruction("OP_MULTIPLY", offset),
        Divide => simple_instruction("OP_DIVIDE", offset),
        Not => simple_instruction("OP_NOT", offset),
        Negate => simple_instruction("OP_NEGATE", offset),
        Print => simple_instruction("OP_PRINT", offset),
        Return => simple_instruction("OP_RETURN", offset),
    }
}

/////////////////////////////////////// Instruction printers ///////////////////////////////////////

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name:>16}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    print!("{name:>16}");

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
