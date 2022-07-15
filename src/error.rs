//! Provides [InterpretationError], the error that most things return.
use thiserror::Error;

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
