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
    pub code: Vec<u8>,
    pub constants: ValueArray,
    pub lines: Vec<usize>,
}

impl Chunk {
    /// Return a new, empty [Chunk].
    pub fn new() -> Self {
        Chunk::default()
    }

    /// Append a single [OpCode] to the chunk.
    pub fn write(&mut self, opcode: OpCode, line: usize) {
        debug_assert_eq!(self.code.len(), self.lines.len());
        self.code.push(opcode as u8);
        self.lines.push(line)
    }

    pub fn write_index(&mut self, index: u8) {
        let last_line = *self
            .lines
            .iter()
            .last()
            .expect("Expected at least one opcode inserted before this");
        self.code.push(index);
        self.lines.push(last_line);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write(value);
        u8::try_from(self.constants.len() - 1).expect("Exceeded size available for u8")
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
}
