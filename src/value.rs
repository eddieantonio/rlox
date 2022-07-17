//! Representation of values in Lox.

extern crate static_assertions as sa;

/// A Lox runtime value.
///
/// Currently, only numbers ([f64]), booleans, and nil are supported.
///
/// You can create a Lox value from its equivalent Rust type:
///
/// ```
/// # use rlox::value::Value;
/// let float: f64 = 0.5;
/// let v: Value = float.into();
/// assert_eq!("0.5".to_owned(), v.to_string());
///
/// let switch = false;
/// let v: Value = switch.into();
/// assert_eq!("false", v.to_string());
/// ```
///
/// This even works with `Option<>`: `None` turns [Value::Nil].
///
/// ```
/// # use rlox::value::Value;
/// let option = Some(0.25);
/// let v: Value = option.into();
/// assert_eq!("0.25", v.to_string());
///
/// let option: Option<f64> = None;
/// let v: Value = option.into();
/// assert_eq!("nil", v.to_string());
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    /// Nil. Doing anything with this is usually an error.
    Nil,
    /// A boolean.
    Boolean(bool),
    /// All numbers in Lox are 64-bit floating point.
    Number(f64),
}
sa::assert_impl_all!(Value: Copy);

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

    /// Returns true if this value is a Lox number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Returns true if this value is "falsy".
    pub fn is_falsy(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }

    /// Applies Lox's rules for equality, returning a Rust bool.
    #[inline]
    pub fn equal(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Number(a), Number(b)) => a == b,
            (Boolean(a), Boolean(b)) => a == b,
            (Nil, Nil) => true,
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
        }
    }
}

// Uninitialized values are nil.
impl Default for Value {
    fn default() -> Value {
        Value::Nil
    }
}

// Convert any Rust float into a Lox value.
impl From<f64> for Value {
    fn from(float: f64) -> Value {
        Value::Number(float)
    }
}

// Convert any Rust float into a Lox value.
impl From<bool> for Value {
    fn from(value: bool) -> Value {
        Value::Boolean(value)
    }
}

// Convert any Rust option of float to a Lox value.
impl From<Option<f64>> for Value {
    fn from(option: Option<f64>) -> Value {
        option.map(Value::Number).unwrap_or(Value::Nil)
    }
}

// Convert any Rust option of bool to a Lox value.
impl From<Option<bool>> for Value {
    fn from(option: Option<bool>) -> Value {
        option.map(Value::Boolean).unwrap_or(Value::Nil)
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
        self.values.get(index).copied()
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
