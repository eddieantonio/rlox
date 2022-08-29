//! A garbage collector (GC) that pretends to have a `'static` lifetime.
//!
//! Normally "GC" stands for "garbage collector", but in this codebase, "GC" just stands for "garbage" 🙃

use std::collections::HashSet;

/// A garbage collector, which is really more of a big store of all dynamic data in the
/// application. For now, it's just string data, and there is no reference counting so all strings
/// are kept forever until the GC is dropped. Right now it literally collects garbage. Forever 😇
#[derive(Clone, Debug, Default)]
pub struct GC {
    strings: HashSet<String>,
}

/// A token that indicates that the global static [GC] has been installed. The only way to obtain
/// this token is to install the GC somehow (for example, by calling [ActiveGC::install]).
/// When this token is dropped, the global static GC will be uninstalled and dropped.
///
/// ```
/// # use rlox::gc::ActiveGC;
/// # use rlox::value::Value;
/// let gc = ActiveGC::install();
///
/// // Now the GC is active and can be used.
/// assert_eq!(0, ActiveGC::n_strings());
///
/// // Strings in Lox **require** the active GC:
/// let lox_string: Value = "hello".into();
/// assert_eq!(1, ActiveGC::n_strings());
/// // when the gc gets dropped (goes out of scope), the GC is automatically uninstalled.
/// ```
#[derive(Debug)]
pub struct ActiveGC(());

/// The actual static (global) [GC] instance. Install with `into_active_gc()`.
static mut ACTIVE_GC: Option<GC> = None;

impl GC {
    /// Adds a string to storage. Returns a reference to the stored string.
    pub fn store_string(&mut self, owned: String) -> &str {
        // HACK: with the current HashMap/HashSet API, I cannot figure out how to do things without
        // a clone 😭
        let key = owned.clone();
        self.strings.insert(owned);
        self.strings.get(&key).unwrap()
    }

    /// Consume self and convert it into the [ActiveGC].
    #[must_use]
    pub fn into_active_gc(self) -> ActiveGC {
        unsafe {
            ACTIVE_GC = Some(self);
        }
        ActiveGC(())
    }

    /// Return how many strings are currently stored.
    fn n_strings(&self) -> usize {
        self.strings.len()
    }
}

impl ActiveGC {
    /// Create a [GC] and install it as the active GC.
    ///
    /// # Panics
    ///
    /// Only one [GC] instance can be active at a time. The process panics
    /// **non-deterministically** if you try to install a [GC] while one is already installed.
    #[must_use]
    pub fn install() -> ActiveGC {
        GC::default().into_active_gc()
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // The following methods these delegate to the active GC instance:
    ///////////////////////////////////////////////////////////////////////////////////////////////

    /// Store a string in the active [GC].
    ///
    /// Returns a reference to the strings storage.
    ///
    /// # Warning
    ///
    /// Note: the reference does not actually have `'static' lifetime. It lives for as long as the
    /// [ActiveGC] is installed.
    pub fn store_string(s: String) -> &'static str {
        Self::get().store_string(s)
    }

    /// Return how many strings are currently stored.
    pub fn n_strings() -> usize {
        Self::get().n_strings()
    }

    /// Get the current active [GC].
    fn get() -> &'static mut GC {
        unsafe { &mut ACTIVE_GC }
            .as_mut()
            .expect("Tried to get active GC, but it's not installed")
    }
}

impl Drop for ActiveGC {
    fn drop(&mut self) {
        // Uninstall the GC by taking ownership of it.
        unsafe {
            ACTIVE_GC
                .take()
                .expect("Trying to drop active GC, but it's not installed")
        };
        // GC dropped here!
    }
}

/// Extends [Vec] with one (1) readable method.
trait VecExtension<T> {
    /// Returns a reference to the last entry added.
    fn ref_to_last_item(&self) -> Option<&T>;
}

impl<T> VecExtension<T> for Vec<T> {
    fn ref_to_last_item(&self) -> Option<&T> {
        self.iter().rev().next()
    }
}

#[cfg(test)]
mod test {
    // Since the active GC is SHARED, MUTABLE STATE 👹, these tests **must** run in serial, or else
    // they will trample over each others' GC :/
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn test_gc() {
        let mut gc = GC::default();
        let original = "hello".to_owned();
        let s = gc.store_string(original);
        assert_eq!("hello", s);
        assert_eq!(1, gc.n_strings());
    }

    #[test]
    #[serial]
    fn test_ownership() {
        let gc = GC::default();
        let _active_gc = gc.into_active_gc();

        let original = "🦀".to_owned();
        let s = ActiveGC::store_string(original);
        assert_eq!("🦀", s);
        assert_eq!(1, ActiveGC::n_strings());
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Tried to get active GC")]
    fn test_using_active_gc_when_not_installed() {
        ActiveGC::store_string("🎷".to_owned());
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Tried to get active GC")]
    fn test_using_active_gc_after_drop() {
        let gc = GC::default();
        {
            let _active_gc = gc.into_active_gc();
            assert_eq!(0, ActiveGC::n_strings());
        }

        ActiveGC::store_string("🍕".to_owned());
    }
}
