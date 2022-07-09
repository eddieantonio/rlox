use rlox::prelude::*;

fn main() -> rlox::vm::Result<()> {
    let mut vm = VM::default();
    let mut c = Chunk::new();
    let i = c.add_constant(1.2);
    c.write_opcode(OpCode::Constant, 123).with_operand(i);
    c.write_opcode(OpCode::Return, 123);

    vm.interpret(&c)
}
