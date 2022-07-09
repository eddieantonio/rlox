use rlox::prelude::*;

fn main() -> rlox::vm::Result<()> {
    let mut vm = VM::default();
    let mut c = Chunk::new();

    // equiv to Lox program:
    // ```
    // return -((1.2 + 3.4) / 5.6);
    // ```
    let constant = c.add_constant(1.2.into());
    c.write_opcode(OpCode::Constant, 1).with_operand(constant);

    let constant = c.add_constant(3.4.into());
    c.write_opcode(OpCode::Constant, 1).with_operand(constant);

    c.write_opcode(OpCode::Add, 1);

    let constant = c.add_constant(5.6.into());
    c.write_opcode(OpCode::Constant, 1).with_operand(constant);

    c.write_opcode(OpCode::Divide, 1);
    c.write_opcode(OpCode::Negate, 1);

    c.write_opcode(OpCode::Return, 1);

    vm.interpret(&c)
}
