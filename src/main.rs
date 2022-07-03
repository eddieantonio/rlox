use rlox::debug::disassemble_chunk;
use rlox::prelude::*;

fn main() {
    let mut c = Chunk::new();
    let i = c.add_constant(1.2);
    c.write(OpCode::Constant, 123);
    c.write_index(i);
    c.write(OpCode::Return, 123);
    disassemble_chunk(&c, "test chunk");
}
