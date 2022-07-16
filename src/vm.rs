//! The bytecode virtual machine.

use crate::compiler;
use crate::prelude::{Chunk, OpCode, Value};

/// Used as the minimum capacity of the stack.
/// Since we're using a growable [Vec], the stack size can be arbitrarily large.
const STACK_SIZE: usize = 256;

/// Maintains state for the Lox virtual machine.
pub struct VM {
    /// Instruction pointer --- index into the chunk for the next opcode to be executed
    // TODO: convert to slice?
    ip: usize,
    /// Value stack -- modified as elements are pushed and popped from the stack.
    stack: Vec<Value>,
}

/// Fetches the next bytecode in the chunk, **AND** increments the instruction pointer.
///
/// Note: use [current_ip] to get the "current" value of the instruction pointer being executed
/// right now.
macro_rules! next_bytecode {
    ($self: ident, $chunk: ident) => {{
        let byte = $chunk.get($self.ip);
        $self.ip += 1;
        byte
    }};
}

/// Gets the value of the current instruction pointer. To be used in conjunction with
/// [next_bytecode].
macro_rules! current_ip {
    ($self: ident) => {
        $self.ip - 1
    };
}

impl VM {
    /// Interpret some the Lox bytecode in the given [Chunk].
    pub fn interpret(&mut self, source: &str) -> crate::Result<()> {
        let chunk = compiler::compile(source)?;
        self.ip = 0;
        self.run(&chunk)
    }

    /// The main opcode interpreter loop.
    fn run(&mut self, chunk: &Chunk) -> crate::Result<()> {
        use OpCode::*;

        loop {
            if cfg!(feature = "trace_execution") {
                use crate::debug::disassemble_instruction;

                // Prints the current stack:
                print!("        ");
                for value in self.stack.iter() {
                    print!("[ {value:?} ]")
                }
                println!();

                // Print the next instruction:
                disassemble_instruction(chunk, self.ip);
            }

            let opcode = next_bytecode!(self, chunk)
                .expect("I have an instruction pointer within range")
                .as_opcode();

            match opcode {
                Some(Constant) => {
                    let constant = next_bytecode!(self, chunk)
                        .expect("there should be an operand")
                        .resolve_constant()
                        .expect("there should be a constant at this index");
                    self.push(constant);
                }
                Some(Add) => self.binary_op(|a, b| a + b),
                Some(Subtract) => self.binary_op(|a, b| a - b),
                Some(Multiply) => self.binary_op(|a, b| a * b),
                Some(Divide) => self.binary_op(|a, b| a / b),
                Some(Negate) => {
                    let value = self.pop();
                    self.push(match value {
                        Value::Number(num) => (-num).into(),
                    });
                }
                Some(Return) => {
                    let return_value = self.pop();
                    println!("{return_value}");

                    return Ok(());
                }
                Some(BranchIfFalsy) => {
                    let Value::Number(value) = self.pop();
                    let offset = next_bytecode!(self, chunk)
                        .expect("there should be an operand")
                        .as_absolute_offset();

                    if value == 0.0 {
                        self.ip = offset;
                    }
                }
                Some(Jump) => {
                    let offset = next_bytecode!(self, chunk)
                        .expect("there should be an operand")
                        .as_absolute_offset();
                    self.ip = offset;
                }
                None => panic!("fetched invalid opcode at {}", current_ip!(self)),
            }
        }
    }

    /// Pops two operands on the stack to perform a binary operation.
    fn binary_op<F>(&mut self, op: F)
    where
        F: Fn(f64, f64) -> f64,
    {
        let rhs = self.pop();
        let lhs = self.pop();

        use Value::Number;
        match (lhs, rhs) {
            (Number(a), Number(b)) => self.push(op(a, b).into()),
        }
    }

    /// Pushes a [Value] on to the value stack.
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pops and returns the top [Value] on the value stack.
    ///
    /// # Panics
    ///
    /// Panics when the value stack is empty. Given well-formed Lox bytecode, a pop cannot occur
    /// when the value stack is empty; therefore the interpreter panics if it is in this state.
    #[inline(always)]
    fn pop(&mut self) -> Value {
        self.stack.pop().expect("value stack is empty")
    }
}

impl Default for VM {
    fn default() -> Self {
        // Create a VM with the value stack pre-allocated to the minimum size.
        VM {
            ip: 0,
            stack: Vec::with_capacity(STACK_SIZE),
        }
    }
}
