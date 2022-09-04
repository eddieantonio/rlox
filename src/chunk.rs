//! Organizes bytecode into a [Chunk] of [OpCode]s.
//!
//! # Example
//!
//! ```
//! use rlox::prelude::*;
//!
//! // Create a chunk:
//! let mut chunk = Chunk::new();
//!
//! // Add a constant to it:
//! if let Some(constant_index) = chunk.add_constant(1.2.into()) {
//!     chunk.write_opcode(OpCode::Constant, 1).with_operand(constant_index);
//!     chunk.write_opcode(OpCode::Return, 1);
//! }
//!
//! // It should be 3 bytes:
//! assert_eq!(3, chunk.len());
//! ```

use crate::value::{Value, ValueArray};
use crate::with_try_from_u8;

with_try_from_u8! {
    /// A one-byte operation code for Lox.
    ///
    /// (See Crafting Interpreters, p. 244)
    #[repr(u8)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum OpCode {
        // Opcodes for constants:
        /// Uses the operand as an index into the constant pool, and pushes that value on to the stack.
        Constant,
        /// Pushes `nil` on the stack.
        Nil,
        /// Pushes `true` on the stack.
        True,
        /// Pushes `false` on the stack.
        False,

        /// Pops the the top of the stack, discarding it forever.
        Pop,

        // Opcodes for dealing with local variables
        /// Uses the operand to index into value stack to find a suitable local variable
        /// and push it onto the stack.
        GetLocal,
        /// Pops the top value from the stack and uses the operand to index into value stack,
        /// and assigns the popped value to the location on the stack.
        SetLocal,
        // Opcodes for dealing with global variables
        /// Uses the operand to the constant pool to find the global name;
        /// Pushes the value of the global onto the stack.
        GetGlobal,
        /// Uses the operand to the constant pool to find the global name;
        /// Pops the top of the stack and assigns it to the global variable indicated by the
        /// operand.
        DefineGlobal,
        /// Uses the operand to the constant pool to find the global name;
        /// Pops the top of the stack and assigns it to the global variable.
        /// The global variable must already exist.
        SetGlobal,

        // Opcodes for expressions and operations
        /// Pops RHS, then LHS; pushes LHS == RHS on to the stack.
        Equal,
        /// Pops RHS, then LHS; pushes LHS > RHS on to the stack.
        Greater,
        /// Pops RHS, then LHS; pushes LHS < RHS on to the stack.
        Less,
        /// Pops RHS, then LHS; pushes LHS + RHS on to the stack.
        Add,
        /// Pops RHS, then LHS; pushes LHS - RHS on to the stack.
        Subtract,
        /// Pops RHS, then LHS; pushes LHS * RHS on to the stack.
        Multiply,
        /// Pops RHS, then LHS; pushes LHS / RHS on to the stack.
        Divide,
        /// Pops the top of the stack; pushes !TOS
        Not,
        /// Pops the top of the stack; pushes -TOS
        Negate,

        // Opcodes for statements:

        /// Pops the top value of the stack and prints it to `stdout`.
        Print,
        /// Pops the top value of the stack and returns from the execution of the current chunk.
        Return,
    }
}

/// A chunk of bytecode, including a constant pool.
///
/// The _byte stream_ contains both [OpCode]s and operands, which are encoded serially, inline.
/// Valid bytes from the byte stream can be obtained using [Chunk::get()].
///
/// Arbitrary bytes **cannot** be written to the byte stream. Instead, one must write an [OpCode]
/// with [Chunk::write_opcode()], and then use the returned [WrittenOpcode] to write additional
/// operands (which may be arbitrary bytes).
///
/// (See Crafting Interpreters, p. 244)
#[derive(Default, Debug)]
pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray,
    lines: Vec<usize>,
}

/// A valid byte from a chunk, obtained using [Chunk::get()].
///
/// You may then apply methods to interpret the byte that is required in the given context.
///
/// # Examples
///
/// ```
/// # use rlox::prelude::*;
/// let mut chunk = Chunk::new();
///
/// // Write a valid program into the chunk:
/// assert_eq!(Some(0), chunk.add_constant(1.0.into()));
/// chunk.write_opcode(OpCode::Constant, 1).with_operand(0);
///
/// // Get a valid byte from the chunk:
/// let byte = chunk.get(0);
/// assert!(byte.is_some());
///
/// // Treat it as an OpCode:
/// let byte = byte.unwrap();
/// assert_eq!(Some(OpCode::Constant), byte.as_opcode());
///
/// // Get another valid byte from the input stream:
/// let byte = chunk.get(1);
/// assert!(byte.is_some());
///
/// // Treat it as a constant index:
/// let byte = byte.unwrap();
/// assert_eq!(0, byte.as_constant_index());
/// ```
#[derive(Clone, Copy)]
pub struct BytecodeEntry<'a> {
    byte: u8,
    provenance: &'a Chunk,
}

/// An [OpCode] that has already been written to the bytestream.
///
/// The byte stream can be augmented with an additional operand.
///
/// # Examples
///
/// ```
/// # use rlox::prelude::*;
/// let mut chunk = Chunk::new();
///
/// // Write an opcode and its operand to the byte stream:
/// chunk.write_opcode(OpCode::Constant, 1).with_operand(0);
/// assert_eq!(2, chunk.len());
/// ```
pub struct WrittenOpcode<'a> {
    line: usize,
    provenance: &'a mut Chunk,
}

///////////////////////////////////////// Implementation //////////////////////////////////////////

impl Chunk {
    /// Return a new, empty [Chunk].
    pub fn new() -> Self {
        Chunk::default()
    }

    /// Get a single byte from the byte stream. See [BytecodeEntry] for how this byte can be
    /// interpreted.
    ///
    /// Returns `Some(entry)` when the offset is in `(0..self.len())`; `None` otherwise.
    pub fn get(&self, offset: usize) -> Option<BytecodeEntry> {
        self.code.get(offset).copied().map(|byte| BytecodeEntry {
            byte,
            provenance: self,
        })
    }

    /// Append a single [OpCode] to the chunk.
    ///
    /// Returns a [WrittenOpcode], which is a handle that can be used to append additional
    /// operands to the byte stream.
    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) -> WrittenOpcode {
        self.write(opcode as u8, line);

        WrittenOpcode {
            line,
            provenance: self,
        }
    }

    /// Adds a constant to the constant pool, and returns its index, if successful.
    ///
    /// # Errors
    ///
    /// A constant index must fit in a [u8]; therefore, **no more than 256 constants may be
    /// added**. This method will return `None` when there are already at least 256 constants
    /// added.
    pub fn add_constant(&mut self, value: Value) -> Option<u8> {
        let index = self.constants.len();
        self.constants.write(value);
        u8::try_from(index).ok()
    }

    /// Returns the line number for whatever is at the given offset.
    pub fn line_number_for(&self, offset: usize) -> Option<usize> {
        self.lines.get(offset).copied()
    }

    /// Returns the length of the byte stream.
    #[inline]
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// Returns true if nothing has been appended to the byte stream.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    /// Actually writes to the byte stream.
    fn write(&mut self, payload: u8, line: usize) {
        debug_assert_eq!(self.code.len(), self.lines.len());
        self.code.push(payload);
        self.lines.push(line)
    }
}

impl<'a> BytecodeEntry<'a> {
    /// Returns the byte interpreted as an index into the constant pool.
    ///
    /// This method never fails, as this method does not check whether the index is a _valid_ index
    /// into the constant pool.
    // TODO: rename this to just "as_index()"
    #[inline(always)]
    pub fn as_constant_index(self) -> usize {
        self.byte as usize
    }

    /// Returns the byte decoded as an [OpCode].
    /// Returns `None` if the byte is not a valid opcode.
    #[inline]
    pub fn as_opcode(self) -> Option<OpCode> {
        self.byte.try_into().ok()
    }

    /// Yanks out a constant from the constant pool.
    ///
    /// Interprets the byte as an index into this entry's [Chunk]'s constant pool, and returns the
    /// assosiated value.
    ///
    /// Returns `Some(value)` if the index is a valid entry in the constants pool. `None`
    /// otherwise.
    ///
    /// # See also
    ///
    ///  - [BytecodeEntry::as_constant_index()]
    #[inline]
    pub fn resolve_constant(self) -> Option<Value> {
        self.provenance.constants.get(self.as_constant_index())
    }

    /// Same as [BytecodeEntry::resolve_constant], but returns (index, value).
    #[inline]
    pub fn resolve_constant_with_index(self) -> Option<(usize, Value)> {
        self.resolve_constant()
            .map(|value| (self.as_constant_index(), value))
    }
}

impl<'a> WrittenOpcode<'a> {
    /// Consumes `self` and appends the operand to the byte stream for the last written instruction.
    #[inline]
    pub fn with_operand(self, index: u8) {
        self.provenance.write(index, self.line);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn boring_test_of_chunk() {
        let c = Chunk::default();
        assert_eq!(0, c.code.len());
    }

    #[test]
    fn mess_around_with_bytecode() {
        let mut c = Chunk::new();
        let i = c.add_constant(1.0.into()).unwrap();
        c.write_opcode(OpCode::Constant, 123).with_operand(i);
        c.write_opcode(OpCode::Return, 123);

        assert!(c.len() >= 3);

        // Constant
        assert_eq!(Some(OpCode::Constant), c.get(0).unwrap().as_opcode());
        let operand = c.get(1);
        assert_eq!(Some(0), operand.map(|b| b.as_constant_index()));
        assert_eq!(Some(1.0.into()), operand.and_then(|b| b.resolve_constant()));

        // Return
        assert_eq!(Some(OpCode::Return), c.get(2).unwrap().as_opcode());
    }
}
