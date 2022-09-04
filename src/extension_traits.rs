//! Provides useful [extension traits][]. Okay, currently just [VecLast].
//!
//! [extension traits]: https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html

/// Extends [Vec] with one (1) readable method.
pub trait VecLast<T> {
    /// Returns a reference to the last entry added.
    fn ref_to_last_item(&self) -> Option<&T>;
}

impl<T> VecLast<T> for Vec<T> {
    fn ref_to_last_item(&self) -> Option<&T> {
        self.iter().rev().next()
    }
}
