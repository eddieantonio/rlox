//! The bytecode virtual machine.

use std::collections::HashMap;

use crate::chunk::BytecodeEntry;
use crate::compiler;
use crate::gc::ActiveGC;
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
    /// The globals in this program.
    globals: HashMap<&'a str, Value>,
    /// We don't access the GC directly, but we need it to live as long as the VM.
    _active_gc: &'a ActiveGC,
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
        let active_gc = ActiveGC::install();
        let chunk = compiler::compile(source, &active_gc)?;
        let mut vm = VmWithChunk {
            ip: 0,
            stack: Vec::with_capacity(STACK_SIZE),
            chunk: &chunk,
            globals: HashMap::default(),
            _active_gc: &active_gc,
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
                if self.stack.is_empty() {
                    print!("<empty>");
                } else {
                    for value in self.stack.iter() {
                        print!("[ {value:?} ]")
                    }
                }
                println!();

                // Print the next instruction:
                disassemble_instruction(chunk, self.ip);
            }

            let opcode = self
                .next_bytecode()
                .expect("I have an instruction pointer within range")
                .as_opcode();

            match opcode {
                Some(Constant) => {
                    let constant = self
                        .next_bytecode()
                        .expect("there should be an operand")
                        .resolve_constant()
                        .expect("there should be a constant at this index");
                    self.push(constant);
                }
                Some(Nil) => self.push(Value::Nil),
                Some(True) => self.push(true.into()),
                Some(False) => self.push(false.into()),
                Some(Pop) => {
                    self.pop();
                }
                Some(GetLocal) => {
                    let slot = self.next_bytecode().expect("operand").as_constant_index();
                    self.push(*self.stack.get(slot).expect("local variable"));
                }
                Some(SetLocal) => {
                    let slot = self.next_bytecode().expect("operand").as_constant_index();
                    let value = self.pop();
                    self.stack[slot] = value;
                }
                Some(GetGlobal) => {
                    let name = self.next_string_constant();
                    match self.globals.get(name) {
                        Some(&value) => self.push(value),
                        None => {
                            let message = format!("undefined global variable: {name}");
                            self.runtime_error(&message)?;
                        }
                    };
                }
                Some(DefineGlobal) => {
                    let name = self.next_string_constant();
                    let value = self.pop();
                    self.globals.insert(name, value);
                }
                Some(SetGlobal) => {
                    let name = self.next_string_constant();
                    let value = self.peek(0);
                    if self.globals.insert(name, value).is_none() {
                        // Tried to assign to an undefined global variable.
                        // First, clean-up the variable we accidentally created...
                        self.globals.remove(name);

                        // THEN, report an error and exit.
                        let message = format!("Undefined variable: '{name}'");
                        self.runtime_error(&message)?;
                    }
                }
                Some(Equal) => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    self.push(lhs.equal(&rhs).into());
                }
                Some(Greater) => self.binary_op(|a, b| a > b)?,
                Some(Less) => self.binary_op(|a, b| a < b)?,
                Some(Add) => {
                    let rhs = self.pop();
                    let lhs = self.pop();

                    match (&lhs, &rhs) {
                        (Value::Number(a), Value::Number(b)) => self.push((a + b).into()),
                        (Value::LoxString(a), Value::LoxString(b)) => {
                            self.push(format!("{a}{b}").into());
                        }
                        _ => self.runtime_error("Can only add numbers or strings")?,
                    }
                }
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
                Some(Print) => {
                    println!("{}", self.pop());
                }
                Some(Return) => {
                    return Ok(());
                }
                None => panic!("fetched invalid opcode at {}", current_ip!(self)),
            }
        }
    }

    /// Raises a runtime error
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
    /// # Panics
    ///
    /// Panics when the value stack is empty. Given well-formed Lox bytecode, a pop cannot occur
    /// when the value stack is empty; therefore the interpreter panics if it is in this state.
    #[inline(always)]
    fn pop(&mut self) -> Value {
        self.stack.pop().expect("value stack is empty")
    }

    /// Peek the nth value on the stack, starting from the top.
    ///
    /// # Panics
    ///
    /// Panics when trying to get a value to far down the stack.
    #[inline(always)]
    fn peek(&self, n: usize) -> Value {
        *self.stack.iter().rev().nth(n).expect("ran off the stack")
    }

    /// Clears the stack.
    #[inline(always)]
    fn reset_stack(&mut self) {
        self.stack.clear()
    }

    /// Fetches the next bytecode in the chunk, **AND** increments the instruction pointer.
    ///
    /// Note: use [current_ip] to get the "current" value of the instruction pointer being executed
    /// right now.
    #[inline]
    fn next_bytecode(&mut self) -> Option<BytecodeEntry<'_>> {
        let byte = self.chunk.get(self.ip);
        self.ip += 1;
        byte
    }

    /// Fetches the next bytecode in the chunk and use it to index the constant pool. The constant
    /// pulled out should be a string (such as global variable name).
    ///
    /// Note: Like [[next_bytecode]], this advances the instruction pointer.
    #[inline]
    fn next_string_constant(&mut self) -> &'static str {
        self.next_bytecode()
            .expect("there should be an operand")
            .resolve_constant()
            .expect("there should be a constant here")
            .to_str()
            .expect("the name must be a string")
    }
}
