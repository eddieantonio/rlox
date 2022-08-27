//! Representation of values in Lox.

use crate::gc::ActiveGC;

extern crate static_assertions as sa;

/// A Lox runtime value.
///
/// Currently, numbers ([f64]), booleans, and nil are supported.
/// To store strings, the global [ActiveGC] **must** be installed.
///
/// You can create a Lox value from its equivalent Rust type:
///
/// ```
/// # use rlox::value::Value;
/// let float: f64 = 0.5;
/// let v: Value = float.into();
/// assert_eq!("0.5", v.to_string());
///
/// let switch = false;
/// let v: Value = switch.into();
/// assert_eq!("false", v.to_string());
/// ```
///
/// This even works with `Option<T>`: `None` turns [Value::Nil].
///
/// ```
/// # use rlox::value::Value;
/// let option = Some(0.25);
/// let v: Value = option.into();
/// assert_eq!("0.25", v.to_string());
///
/// let option = Some(true);
/// let v: Value = option.into();
/// assert_eq!("true", v.to_string());
///
/// let option: Option<f64> = None;
/// let v: Value = option.into();
/// assert_eq!("nil", v.to_string());
/// ```
///
/// # Strings
///
/// String data is owned and stored in the current [ActiveGC].  If an [ActiveGC] is not installed,
/// the process will panic since there is nowhere to store the string data.
///
/// ```
/// # use rlox::gc::ActiveGC;
/// # use rlox::value::Value;
/// let _gc = rlox::gc::ActiveGC::install();
/// let string = "Hello".to_owned();
/// let v: Value = string.into();
/// assert_eq!(true, v.is_string());
/// assert_eq!(false, v.is_falsy());
/// // _gc will be dropped, deallocating the GC and all strings it owns
/// ```
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Value {
    /// Nil. Doing anything with this is usually an error.
    #[default]
    Nil,
    /// A boolean.
    Boolean(bool),
    /// All numbers in Lox are 64-bit floating point.
    Number(f64),
    /// Strings (the owned contents belong to the [ActiveGC])
    LoxString(&'static str),
}

/// A collection of values. Useful for a constant pool.
#[derive(Default, Debug, Clone)]
pub struct ValueArray {
    // TODO: I copied the book, but I'm not convinced this struct is better than just a Vec<Value>.
    values: Vec<Value>,
}

///////////////////////////////////////// Implementation //////////////////////////////////////////

impl Value {
    /// Returns true if this value is a Lox boolean.
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Returns true if this value is a Lox's nil.
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Returns true if this value is a Lox object.
    pub fn is_obj(&self) -> bool {
        unimplemented!("object types don't exist yet");
    }

    /// Returns true if this value is a Lox number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Returns true if this value is a Lox string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::LoxString(_))
    }

    /// Returns true if this value is "falsy".
    pub fn is_falsy(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }

    /// Returns a reference to the string contents, if this value is a Lox string.
    pub fn to_str(&self) -> Option<&str> {
        match self {
            Value::LoxString(string) => Some(string),
            _ => None,
        }
    }

    /// Applies Lox's rules for equality, returning a Rust bool.
    #[inline]
    pub fn equal(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Number(a), Number(b)) => a == b,
            (Boolean(a), Boolean(b)) => a == b,
            (Nil, Nil) => true,
            (LoxString(a), LoxString(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(num) => write!(f, "{num}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::LoxString(string) => write!(f, "{string}"),
        }
    }
}

// Convert any Rust float into a Lox value.
impl From<f64> for Value {
    #[inline(always)]
    fn from(float: f64) -> Value {
        Value::Number(float)
    }
}

// Convert any Rust float into a Lox value.
impl From<bool> for Value {
    #[inline(always)]
    fn from(value: bool) -> Value {
        Value::Boolean(value)
    }
}

// Convert any Rust (owned) string to a Lox value.
impl From<String> for Value {
    fn from(owned: String) -> Value {
        let reference = ActiveGC::store_string(owned);
        Value::LoxString(reference)
    }
}

// Copy any Rust (borrowed) string to a Lox value.
impl From<&str> for Value {
    fn from(borrowed: &str) -> Value {
        let reference = ActiveGC::store_string(borrowed.to_owned());
        Value::LoxString(reference)
    }
}

// Convert any Rust option of float to a Lox value.
impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    #[inline]
    fn from(option: Option<T>) -> Value {
        option.map(Into::into).unwrap_or(Value::Nil)
    }
}

impl ValueArray {
    /// Return an empty [ValueArray].
    pub fn new() -> Self {
        ValueArray::default()
    }

    /// Returns a [Value] at the given index. If the index is out of bounds, this returns `None`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<Value> {
        self.values.get(index).cloned()
    }

    /// Add a new [Value] to the array
    pub fn write(&mut self, value: Value) {
        // TODO: the book returns the index that this was written... should we do the same here?
        self.values.push(value)
    }

    /// Returns how many values are in the pool.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if there are no values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
