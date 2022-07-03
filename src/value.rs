/// The only kind of value for now:
pub type Value = f64;

/// A whole bunch of values, slapped together. Useful for a constant pool.
#[derive(Default)]
pub struct ValueArray {
    pub values: Vec<Value>,
}

impl ValueArray {
    /// Return an empty [ValueArray].
    pub fn new() -> Self {
        ValueArray::default()
    }

    /// Add a new [Value] to the array
    pub fn write(&mut self, value: Value) {
        // TODO: the book returns the index that this was written... should we do the same here?
        self.values.push(value)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
