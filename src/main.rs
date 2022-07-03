use rlox::debug::disassemble_chunk;
use rlox::prelude::*;

fn main() {
    let mut c = Chunk::new();
    let i = c.add_constant(1.2);
    c.write_opcode(OpCode::Constant, 123).with_operand(i);
    c.write_opcode(OpCode::Return, 123);
    disassemble_chunk(&c, "test chunk");
}
