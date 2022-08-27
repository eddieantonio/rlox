//! Garbage Collector collector (GC) with `'static` lifetime.
//! Normally "GC" stands for "garbage collector", but in this codebase, it's just "garbage" 🙃

#[derive(Clone, Debug, Default)]
pub struct GC {
    strings: Vec<String>,
}

/// A token that indicates that the global static [GC] has been installed.
/// When this token is dropped, the global static GC will be uninstalled and dropped.
#[derive(Debug)]
pub struct ActiveGC(());

/// The actual static [GC] instance. Install with `into_active_gc()`.
static mut ACTIVE_GC: Option<GC> = None;

impl GC {
    /// Adds a string to storage.
    pub fn store_string(&mut self, s: String) -> &str {
        self.strings.push(s);
        self.strings.iter().rev().next().as_ref().unwrap()
    }

    /// Consume self and convert it into the active [GC].
    #[must_use]
    pub fn into_active_gc(self) -> ActiveGC {
        unsafe {
            ACTIVE_GC = Some(self);
        }
        ActiveGC(())
    }

    #[cfg(test)]
    fn n_strings(&self) -> usize {
        self.strings.len()
    }
}

impl ActiveGC {
    fn get() -> &'static mut GC {
        unsafe { &mut ACTIVE_GC }
            .as_mut()
            .expect("Tried to get active GC, but it's not installed")
    }

    // All of these delegate to the active [GC] instance:

    pub fn store_string(s: String) -> &'static str {
        Self::get().store_string(s)
    }

    #[cfg(test)]
    fn n_strings() -> usize {
        Self::get().n_strings()
    }
}

impl Drop for ActiveGC {
    fn drop(&mut self) {
        // Uninstall the GC by taking ownership.
        unsafe {
            ACTIVE_GC
                .take()
                .expect("Trying to drop active GC, but it's not installed")
        };
        // GC dropped here!
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gc() {
        let mut gc = GC::default();
        let original = "hello".to_owned();
        let s = gc.store_string(original);
        assert_eq!("hello", s);
        assert_eq!(1, gc.n_strings());
    }

    #[test]
    fn test_ownership() {
        let gc = GC::default();
        let _active_gc = gc.into_active_gc();

        let original = "🦀".to_owned();
        let s = ActiveGC::store_string(original);
        assert_eq!("🦀", s);
        assert_eq!(1, ActiveGC::n_strings());
    }

    #[test]
    #[should_panic]
    fn test_using_active_gc_when_not_installed() {
        ActiveGC::store_string("🎷".to_owned());
    }

    #[test]
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
