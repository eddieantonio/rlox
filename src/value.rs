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
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Nil. Doing anything with this is usually an error.
    Nil,
    /// A boolean.
    Boolean(bool),
    /// All numbers in Lox are 64-bit floating point.
    Number(f64),
    /// Instances and strings
    Object(Obj),
}

/// A Lox object
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Obj {
    // TODO: there should be an trait for Obj that grants access to common fields.
    contents: ObjType,
}

/// What kind of object we can have.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjType {
    LoxString(String),
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
        matches!(self, Value::Object(_))
    }

    /// Returns true if this value is a Lox number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Returns true if this value is a Lox string.
    pub fn is_string(&self) -> bool {
        matches!(
            self,
            // XXX: this data structure is terrible!
            Value::Object(Obj {
                contents: ObjType::LoxString(_)
            })
        )
    }

    /// Returns true if this value is "falsy".
    pub fn is_falsy(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }

    /// Returns a reference to the string contents, if this value is a Lox string.
    pub fn to_str(&self) -> Option<&str> {
        match self {
            Value::Object(Obj {
                contents: ObjType::LoxString(string),
            }) => Some(string),
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
            (Object(_), Object(_)) => self
                .to_str()
                .zip(other.to_str())
                .map(|(a, b)| a == b)
                .unwrap_or(false),
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
            Value::Object(obj) => obj.fmt(f),
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
        Value::Object(Obj {
            contents: ObjType::LoxString(owned),
        })
    }
}

// Copy any Rust (borrowed) string to a Lox value.
impl From<&str> for Value {
    fn from(borrowed: &str) -> Value {
        Value::Object(Obj {
            contents: ObjType::LoxString(borrowed.to_string()),
        })
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

impl std::fmt::Display for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Obj {
                contents: ObjType::LoxString(x),
            } => write!(f, "{}", x),
        }
    }
}
