use crate::prelude::*;

pub fn compile(source: &str) -> Chunk {
    let scanner = Scanner::new(source);

    // Temporary code to scan the entire file and print it.
    let mut line = 0;
    for token in scanner {
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        println!("{:2?} '{}'", token.ttype, token.lexeme);

        if token.ttype == TokenType::Eof {
            break;
        }
    }

    // Create a valid no-op program
    // Note: every program MUST return a valid Lox value!
    let mut chunk = Chunk::default();
    chunk.add_constant(0.0.into());
    chunk.write_opcode(OpCode::Constant, 0).with_operand(0);
    chunk.write_opcode(OpCode::Return, 1);
    chunk
}
