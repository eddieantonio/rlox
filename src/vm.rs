//! The bytecode virtual machine.
use thiserror::Error;

use crate::prelude::{Chunk, OpCode, Value};

/// Used as the minimum capacity of the stack.
/// Since we're using a growable [Vec], the stack size can be arbitrarily large.
const STACK_SIZE: usize = 256;

/// The type returned by [VM::interpret].
pub type Result<T> = std::result::Result<T, InterpretationError>;

/// Maintains state for the Lox virtual machine.
pub struct VM<'a> {
    /// Code to execute
    // TODO: I'm not confident this needs to be in here...?
    // TODO: In Rust, this kind of just makes things more annoying.
    chunk: Option<&'a Chunk>,
    /// Instruction pointer --- index into the chunk for the next opcode to be executed
    // TODO: convert to slice?
    ip: usize,
    /// Value stack -- modified as elements are pushed and popped from the stack.
    stack: Vec<Value>,
}

/// Any error that can occur during interpretation.
#[derive(Debug, Error)]
pub enum InterpretationError {
    /// A compile-time error, such as a syntax error, or a name error.
    #[error("compile-time error")]
    CompileError,
    /// A runtime error, such as a type error or exception.
    #[error("runtime error")]
    RuntimeError,
    // TODO: add a variant for "invalid bytecode"?
}

impl<'a> VM<'a> {
    /// Interpret some the Lox bytecode in the given [Chunk].
    pub fn interpret(&'a mut self, chunk: &'a Chunk) -> Result<()> {
        self.chunk = Some(chunk);
        self.ip = 0;

        self.run()
    }

    /// The main opcode interpreter loop.
    fn run(&mut self) -> Result<()> {
        use OpCode::*;
        let chunk = self.chunk.expect("I should have a valid chunk right now");

        loop {
            if cfg!(feature = "trace_execution") {
                print!("        ");
                for value in self.stack.iter() {
                    print!("[ {value:?} ]")
                }
                println!();
                trace_instruction(chunk, self.ip);
            }

            let opcode = chunk
                .get(self.ip)
                .expect("I have an instruction pointer within range")
                .as_opcode();

            match opcode {
                Some(Constant) => {
                    let constant = chunk
                        .get(self.ip + 1)
                        .expect("there should be an operand")
                        .resolve_constant()
                        .expect("there should be a constant at this index");
                    self.push(constant);
                    self.ip += 2;
                }
                Some(Add) => self.binary_op(|a, b| a + b),
                Some(Subtract) => self.binary_op(|a, b| a - b),
                Some(Multiply) => self.binary_op(|a, b| a * b),
                Some(Divide) => self.binary_op(|a, b| a / b),
                Some(Negate) => {
                    let value = self.pop().expect("value stack is empty");
                    self.push(match value {
                        Value::Number(num) => (-num).into(),
                    });
                    self.ip += 1;
                }
                Some(Return) => {
                    let return_value = self.pop().expect("value stack is empty");
                    println!("{return_value}");

                    return Ok(());
                }
                None => panic!("tried to get an invalid opcode at {}", self.ip),
            }
        }
    }

    /// Pops two operands on the stack to perform a binary operation.
    fn binary_op<F>(&mut self, op: F)
    where
        F: Fn(f64, f64) -> f64,
    {
        let rhs = self.pop().expect("value stack empty (right-hand side)");
        let lhs = self.pop().expect("value stack empty (left-hand side)");

        use Value::Number;
        match (lhs, rhs) {
            (Number(a), Number(b)) => self.push(op(a, b).into()),
        }

        self.ip += 1;
    }

    /// Pushes a [Value] on to the value stack.
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pops and returns the top [Value] on the value stack.
    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }
}

impl<'a> Default for VM<'a> {
    fn default() -> Self {
        // Create a VM with the value stack pre-allocated to the minimum size.
        VM {
            chunk: None,
            ip: 0,
            stack: Vec::with_capacity(STACK_SIZE),
        }
    }
}

#[cfg(feature = "trace_execution")]
fn trace_instruction(chunk: &Chunk, ip: usize) {
    use crate::debug::disassemble_instruction;
    disassemble_instruction(chunk, ip);
}

#[cfg(not(feature = "trace_execution"))]
#[inline(always)]
fn trace_instruction(_chunk: &Chunk, _ip: usize) {
    // do nothing
}
