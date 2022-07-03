//! Contains a [Chunk] of [OpCode].

use crate::value::{Value, ValueArray};

/// A one-byte operation code for Lox.
///
/// (See Crafting Interpreters, p. 244)
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpCode {
    Return,
    Constant,
}

/// A chunk of code, with metadata.
///
/// (See Crafting Interpreters, p. 244)
#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    pub constants: ValueArray,
    lines: Vec<LineNumberRun>,
}

/// A valid byte from a chunk. This byte can then be interpreted as required.
#[derive(Clone, Copy)]
pub struct BytecodeEntry<'a> {
    byte: u8,
    provenance: &'a Chunk,
}

/// An [Opcode] that has already been written to the bytestream.
///
/// This opcode can be augmented with an additional operand.
pub struct WrittenOpcode<'a> {
    line: usize,
    provenance: &'a mut Chunk,
}

/// An entry of run-length encoded line numbers.
/// Every entry signifies that the next [length] bytes have the same line number
#[derive(Debug, Clone)]
struct LineNumberRun {
    /// The actual line number
    line_number: usize,
    /// The starting index in the table
    length: usize,
}

///////////////////////////////////////// Implementation //////////////////////////////////////////

impl Chunk {
    /// Return a new, empty [Chunk].
    pub fn new() -> Self {
        Chunk::default()
    }

    /// Get an entry from the bytecode stream.
    ///
    /// Returns `Some(entry)` when the offset is in [0, self.len()).
    pub fn get(&self, offset: usize) -> Option<BytecodeEntry> {
        self.code.get(offset).copied().map(|byte| BytecodeEntry {
            byte,
            provenance: self,
        })
    }

    /// Append a single [OpCode] to the chunk.
    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) -> WrittenOpcode {
        self.write(opcode as u8, line);

        WrittenOpcode {
            line,
            provenance: self,
        }
    }

    /// Adds a constant to the constant pool, and returns its index.
    ///
    /// # Panics
    ///
    /// Panics when adding the 257th constant or greater. Since the available indices are 0-255,
    /// there is only room for 256 constants. Trying to add more than this will panic.
    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write(value);
        u8::try_from(self.constants.len() - 1).expect("Exceeded size available for u8")
    }

    /// Returns the line number for whatever is at the given offset.
    pub fn line_number_for(&self, offset: usize) -> Option<usize> {
        let mut base_offset = 0;
        for run in self.lines.iter() {
            if (base_offset..base_offset + run.length).contains(&offset) {
                return Some(run.line_number);
            }

            base_offset += run.length;
        }

        None
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
    fn write(&mut self, payload: u8, line_number: usize) {
        self.code.push(payload);

        // Figure out the line number
        if let Some(run) = self.previous_line_number_run() {
            if run.line_number == line_number {
                run.increment()
            } else {
                // Must create new run
                self.lines.push(LineNumberRun::new(line_number))
            }
        } else {
            assert!(self.lines.is_empty());
            self.lines.push(LineNumberRun::new(line_number))
        }
    }

    /// Return the last line number run
    #[inline(always)]
    fn previous_line_number_run(&mut self) -> Option<&mut LineNumberRun> {
        self.lines.iter_mut().rev().next()
    }
}

impl LineNumberRun {
    fn new(line_number: usize) -> Self {
        Self {
            line_number,
            length: 1,
        }
    }

    fn increment(&mut self) {
        self.length += 1;
    }
}

impl<'a> BytecodeEntry<'a> {
    /// Returns the byte as an index into the constant pool.
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
    #[inline]
    pub fn resolve_constant(self) -> Option<Value> {
        self.provenance
            .constants
            .values
            .get(self.as_constant_index())
            .copied()
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

impl TryFrom<u8> for OpCode {
    // TODO: temporary
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            x if x == OpCode::Return as u8 => Ok(OpCode::Return),
            x if x == OpCode::Constant as u8 => Ok(OpCode::Constant),
            _ => Err(()),
        }
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
        let i = c.add_constant(1.0);
        c.write_opcode(OpCode::Constant, 123).with_operand(i);
        c.write_opcode(OpCode::Return, 123);

        assert!(c.len() >= 3);

        // Constant
        assert_eq!(Some(OpCode::Constant), c.get(0).unwrap().as_opcode());
        assert_eq!(Some(0), c.get(1).map(|b| b.as_constant_index()));
        assert_eq!(Some(1.0), c.get(1).and_then(|b| b.resolve_constant()));

        // Return
        assert_eq!(Some(OpCode::Return), c.get(2).unwrap().as_opcode());
    }

    #[test]
    fn line_numbers() {
        let mut c = Chunk::new();

        let idx = c.add_constant(1.2);

        // Write a bunch of opcodes on the same line.
        c.write_opcode(OpCode::Constant, 1).with_operand(idx);
        c.write_opcode(OpCode::Constant, 1).with_operand(idx);
        c.write_opcode(OpCode::Constant, 1).with_operand(idx);
        assert_eq!(6, c.len());

        // Write a bunch of opcodes on a different line.
        c.write_opcode(OpCode::Constant, 2).with_operand(idx);
        c.write_opcode(OpCode::Constant, 2).with_operand(idx);
        c.write_opcode(OpCode::Constant, 2).with_operand(idx);
        c.write_opcode(OpCode::Constant, 2).with_operand(idx);
        assert_eq!(14, c.len());

        // Write an opcode on yet a different line
        c.write_opcode(OpCode::Return, 4);
        assert_eq!(15, c.len());

        // Check line numbers.
        assert_eq!(Some(1), c.line_number_for(2));
        assert_eq!(Some(2), c.line_number_for(10));
        assert_eq!(Some(4), c.line_number_for(c.len() - 1));
    }
}
