//! Provides useful [extension traits][]. Okay, currently just [VecLast].
//!
//! [extension traits]: https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html

/// Extends [Vec] with [VecLast::last()] and [VecLast::last_mut()] methods.
///
/// ```
/// use rlox::extension_traits::VecLast;
///
/// let mut v = vec![1, 2, 3];
/// assert_eq!(Some(&3), v.last());
///
/// v.push(4);
/// assert_eq!(Some(&4), v.last());
///
/// *v.last_mut().unwrap() = 42;
/// assert_eq!(vec![1, 2, 3, 42], v);
///
/// // Both methods return None for an empty list:
/// let mut empty: Vec<()> = vec![];
/// assert_eq!(None, empty.last());
/// assert_eq!(None, empty.last_mut());
/// ```
pub trait VecLast<T> {
    /// Returns a reference to the last item.
    ///
    /// ```
    /// use rlox::extension_traits::VecLast;
    /// assert_eq!(Some(&3), vec![1, 2, 3].last());
    /// ```
    fn last(&self) -> Option<&T>;

    /// Returns a mutable reference to the last item.
    ///
    /// ```
    /// use rlox::extension_traits::VecLast;
    ///
    /// let mut v = vec![0];
    /// *v.last_mut().unwrap() = 1337;
    /// assert_eq!(vec![1337], v);
    /// ```
    fn last_mut(&mut self) -> Option<&mut T>;
}

impl<T> VecLast<T> for Vec<T> {
    fn last(&self) -> Option<&T> {
        self.iter().rev().next()
    }

    fn last_mut(&mut self) -> Option<&mut T> {
        self.iter_mut().rev().next()
    }
}
