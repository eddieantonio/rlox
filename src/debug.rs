use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(c: &Chunk, name: &str) {
    println!("== {name} ==");

    // TODO: rust-ify this (use an iterator?)
    let mut offset = 0;
    while offset < c.code.len() {
        offset = disassemble_instruction(c, offset);
    }
}

pub fn disassemble_instruction(c: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0
        && c.lines
            .get(offset)
            .zip(c.lines.get(offset - 1))
            .map(|(current_line, previous_line)| current_line == previous_line)
            .unwrap()
    {
        print!("   | ");
    } else {
        let line_no = c.lines.get(offset).unwrap();
        print!("{line_no:4} ")
    }

    let byte = *c.code.get(offset).expect("invalid chunk offset");
    let instruction: OpCode = byte.try_into().expect("Invalid instruction");

    use OpCode::*;
    #[allow(unreachable_patterns)]
    match instruction {
        // This is kind of silly in Rust, tbh
        Return => simple_instruction("OP_RETURN", offset),
        Constant => constant_instruction("OP_CONSTANT", c, offset),
        _ => {
            println!("Unknown opcode {instruction:?}");
            offset + 1
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name:>14}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    print!("{name:>14}");

    let index = *chunk.code.get(offset + 1).expect("Chunk too short") as usize;

    let value = chunk
        .constants
        .values
        .get(index)
        .expect("Invalid constant index");

    println!("{index:4} '{value:?}'");

    offset + 2
}
