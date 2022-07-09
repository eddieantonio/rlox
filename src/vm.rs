use thiserror::Error;

use crate::prelude::{Chunk, OpCode, Value};

const STACK_SIZE: usize = 256;

pub type Result<T> = std::result::Result<T, InterpretationError>;

pub struct VM<'a> {
    /// Code to execute
    chunk: Option<&'a Chunk>,
    /// Instruction pointer --- index into the chunk for the next opcode to be executed
    // TODO: convert to slice?
    ip: usize,
    /// Value stack -- modified as elements are pushed and popped from the stack.
    stack: Vec<Value>,
}

#[derive(Debug, Error)]
pub enum InterpretationError {
    #[error("compile-time error")]
    CompileError,
    #[error("runtime error")]
    RuntimeError,
}

impl<'a> VM<'a> {
    pub fn interpret(&'a mut self, chunk: &'a Chunk) -> Result<()> {
        self.chunk = Some(chunk);
        self.ip = 0;

        self.run()
    }

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

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }
}

impl<'a> Default for VM<'a> {
    fn default() -> Self {
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
