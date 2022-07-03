//! rlox main module.

pub mod chunk;
pub mod debug;
pub mod value;

mod with_try_from_u8;

/// Re-exports common items.
pub mod prelude {
    pub use crate::chunk::{Chunk, OpCode};
    pub use crate::value::Value;
}
