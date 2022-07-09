use thiserror::Error;

use crate::prelude::{Chunk, OpCode};

pub type Result<T> = std::result::Result<T, InterpretationError>;

#[derive(Default)]
pub struct VM<'a> {
    chunk: Option<&'a Chunk>,
    /// Instruction pointer --- index into the chunk for the next opcode to be executed
    // TODO: convert to slice?
    ip: usize,
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
            trace_instruction(chunk, self.ip);
            let opcode = chunk
                .get(self.ip)
                .expect("I have an instruction pointer within range")
                .as_opcode();

            match opcode {
                Some(Return) => return Ok(()),
                Some(Constant) => {
                    let constant = chunk
                        .get(self.ip + 1)
                        .expect("there should be an operand")
                        .resolve_constant()
                        .expect("there should be a constant at this index");
                    println!("{constant}");
                    self.ip += 2;
                }
                None => panic!("tried to get an invalid opcode at {}", self.ip),
            }
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
