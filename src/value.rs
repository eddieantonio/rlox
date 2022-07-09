//! Representation of values in Lox.

extern crate static_assertions as sa;

/// Any valid Lox value.
///
/// Currently, only numbers ([f64]) are supported.
///
/// You can create a Lox value from some Rust types:
///
/// ```
/// # use rlox::value::Value;
/// let v: Value = 4.0f64.into();
/// assert_eq!("4".to_owned(), v.to_string());
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Number(f64),
}
sa::assert_impl_all!(Value: Copy);

/// A collection of values. Useful for a constant pool.
#[derive(Default, Debug, Clone)]
pub struct ValueArray {
    // TODO: I copied the book, but I'm not convinced this struct is better than just a Vec<Value>.
    values: Vec<Value>,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Number(num) => write!(f, "{num}"),
        }
    }
}

// Convert any Rust float into a Lox value.
impl From<f64> for Value {
    fn from(float: f64) -> Value {
        Value::Number(float)
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
