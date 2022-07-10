//! A bytecode interpreter for [Lox][lox].
//!
//! See [part III][bytecode] of [Crafting Interpreters][book].
//!
//! [book]: https://craftinginterpreters.com/
//! [bytecode]: https://craftinginterpreters.com/a-bytecode-virtual-machine.html
//! [lox]: https://craftinginterpreters.com/the-lox-language.html

pub mod chunk;
pub mod compiler;
pub mod debug;
pub mod scanner;
pub mod value;
pub mod vm;

mod with_try_from_u8;

/// Re-exports common items.
///
/// Since Part III of Crafting Interpreters is written in C, which lacks explicit features for
/// scoping across modules, many items are assumed to be globally-visible. Therefore, we export the
/// most common "global" items here:
pub mod prelude {
    pub use crate::chunk::{Chunk, OpCode};
    pub use crate::scanner::{Scanner, Token, TokenType};
    pub use crate::value::Value;
    pub use crate::vm::{InterpretationError, VM};
}
