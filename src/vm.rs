//! The bytecode virtual machine.

use crate::compiler;
use crate::prelude::{Chunk, InterpretationError, OpCode, Value};

/// Used as the minimum capacity of the stack.
/// Since we're using a growable [Vec], the stack size can be arbitrarily large.
const STACK_SIZE: usize = 256;

/// Maintains state for the Lox virtual machine.
#[derive(Default)]
pub struct VM {
    // In order to match the interface in Crafting Interpreters, I created this struct.
    // However, it's inconvenient in Rust because of chunk possibly being None; however, we know
    // that there's a state in which the VM MUST have a chunk, which is why VmWithChunk exists.
}

/// A VM with an active chunk
struct VmWithChunk<'a> {
    /// Instruction pointer --- index into the chunk for the next opcode to be executed
    // TODO: convert to slice?
    ip: usize,
    /// Value stack -- modified as elements are pushed and popped from the stack.
    stack: Vec<Value>,
    chunk: &'a Chunk,
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
        let mut vm = VmWithChunk {
            ip: 0,
            stack: Vec::with_capacity(STACK_SIZE),
            chunk: &chunk,
        };
        vm.run()
    }
}

impl<'a> VmWithChunk<'a> {
    /// The main opcode interpreter loop.
    fn run(&mut self) -> crate::Result<()> {
        use OpCode::*;
        let chunk = self.chunk;

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
                Some(Nil) => self.push(Value::Nil),
                Some(True) => self.push(true.into()),
                Some(False) => self.push(false.into()),
                Some(Equal) => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    self.push(lhs.lox_equal(&rhs).into());
                }
                Some(Greater) => self.binary_op(|a, b| a > b)?,
                Some(Less) => self.binary_op(|a, b| a < b)?,
                Some(Add) => self.binary_op(|a, b| a + b)?,
                Some(Subtract) => self.binary_op(|a, b| a - b)?,
                Some(Multiply) => self.binary_op(|a, b| a * b)?,
                Some(Divide) => self.binary_op(|a, b| a / b)?,
                Some(Not) => {
                    let value = self.pop();
                    self.push(value.is_falsy().into());
                }
                Some(Negate) => {
                    if let Value::Number(number) = self.pop() {
                        self.push((-number).into());
                    } else {
                        // TODO: rephrase to remove "compiler-speak" from error message:
                        self.runtime_error("Operand must be a number")?;
                    }
                }
                Some(Return) => {
                    let return_value = self.pop();
                    println!("{return_value}");

                    return Ok(());
                }
                None => panic!("fetched invalid opcode at {}", current_ip!(self)),
            }
        }
    }

    fn runtime_error<T>(&mut self, message: &str) -> crate::Result<T> {
        eprintln!("{message}");

        let line = self.chunk.line_number_for(self.ip).expect("line number");
        eprintln!("[line {line}] in script");

        self.reset_stack();

        Err(InterpretationError::RuntimeError)
    }

    /// Pops two operands on the stack to perform a binary operation.
    fn binary_op<F, T>(&mut self, op: F) -> crate::Result<()>
    where
        F: Fn(f64, f64) -> T,
        T: Into<Value>,
    {
        let rhs = self.pop();
        let lhs = self.pop();

        use Value::Number;
        match (lhs, rhs) {
            (Number(a), Number(b)) => self.push(op(a, b).into()),
            (_, _) => self.runtime_error("Operands must be numbers")?,
        };

        Ok(())
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

    /// Peeks at the value relative to the top of the stack.
    ///
    /// # Panics
    ///
    ///  * When the stack is empty
    ///  * When the distance goes off the end of the stack
    #[inline(always)]
    #[allow(dead_code)]
    fn peek(&self, distance: usize) -> Value {
        // Copied this code from Crafting Interpreters, but I'm not sure how useful it will be.
        *self
            .stack
            .get(self.stack.len() - 1 - distance)
            .expect("peeked escaped bounds of the stack")
    }

    #[inline(always)]
    fn reset_stack(&mut self) {
        self.stack.clear()
    }
}
