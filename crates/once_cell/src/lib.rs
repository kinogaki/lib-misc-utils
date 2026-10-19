//! Kinogaki's own `once_cell`, published under the crates.io name so the
//! frozen naga, wgpu-core and read-fonts resolve to it through
//! `[patch.crates-io]`. It is not upstream once_cell: it has the three types
//! those crates use, as thin wrappers over std's `OnceLock` and `LazyLock`
//! (which grew out of once_cell).

pub mod sync {
    //! Thread-safe cells initialised once.

    use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};

    /// A cell written once (`set` or an initialiser), then only read.
    ///
    /// Initialisers run one at a time under `init` (std's `OnceLock` has no
    /// stable fallible initialiser), and the value lives in the `OnceLock`, so
    /// reads after initialisation take no lock.
    pub struct OnceCell<T> {
        value: OnceLock<T>,
        init: Mutex<()>,
    }

    impl<T> OnceCell<T> {
        pub const fn new() -> OnceCell<T> {
            OnceCell {
                value: OnceLock::new(),
                init: Mutex::new(()),
            }
        }

        pub fn get(&self) -> Option<&T> {
            self.value.get()
        }

        pub fn get_mut(&mut self) -> Option<&mut T> {
            self.value.get_mut()
        }

        /// Store `value` unless the cell is already set; `Err(value)` then.
        pub fn set(&self, value: T) -> Result<(), T> {
            let _init = self.init.lock().unwrap_or_else(PoisonError::into_inner);
            self.value.set(value)
        }

        /// The value, computing it with `init` if unset (other threads wait).
        pub fn get_or_init<F: FnOnce() -> T>(&self, init: F) -> &T {
            match self.get_or_try_init(|| Ok::<T, core::convert::Infallible>(init())) {
                Ok(value) => value,
                Err(never) => match never {},
            }
        }

        /// The value, computing it with `init` if unset; an error leaves the
        /// cell unset for a later attempt.
        pub fn get_or_try_init<F: FnOnce() -> Result<T, E>, E>(&self, init: F) -> Result<&T, E> {
            if let Some(value) = self.value.get() {
                return Ok(value);
            }
            let _init = self.init.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(value) = self.value.get() {
                return Ok(value);
            }
            let value = init()?;
            Ok(self.value.get_or_init(|| value))
        }

        pub fn into_inner(self) -> Option<T> {
            self.value.into_inner()
        }

        pub fn take(&mut self) -> Option<T> {
            self.value.take()
        }
    }

    impl<T> Default for OnceCell<T> {
        fn default() -> OnceCell<T> {
            OnceCell::new()
        }
    }

    impl<T: Clone> Clone for OnceCell<T> {
        fn clone(&self) -> OnceCell<T> {
            let out = OnceCell::new();
            if let Some(value) = self.get() {
                let _ = out.set(value.clone());
            }
            out
        }
    }

    impl<T: core::fmt::Debug> core::fmt::Debug for OnceCell<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_tuple("OnceCell").field(&self.get()).finish()
        }
    }

    /// A value computed on first access.
    pub struct Lazy<T, F = fn() -> T>(LazyLock<T, F>);

    impl<T, F: FnOnce() -> T> Lazy<T, F> {
        pub const fn new(init: F) -> Lazy<T, F> {
            Lazy(LazyLock::new(init))
        }

        /// The value, computing it first if needed.
        pub fn force(this: &Lazy<T, F>) -> &T {
            LazyLock::force(&this.0)
        }
    }

    impl<T, F: FnOnce() -> T> core::ops::Deref for Lazy<T, F> {
        type Target = T;
        fn deref(&self) -> &T {
            &self.0
        }
    }

    impl<T: core::fmt::Debug, F: FnOnce() -> T> core::fmt::Debug for Lazy<T, F> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_tuple("Lazy").field(&**self).finish()
        }
    }
}

pub mod race {
    //! Upstream's lock-free cells; here they wait like `OnceLock`, which only
    //! changes who computes the value when two threads race to initialise it.

    use std::sync::OnceLock;

    /// A once-written cell holding a `Box<T>`.
    #[derive(Default)]
    pub struct OnceBox<T>(OnceLock<Box<T>>);

    impl<T> OnceBox<T> {
        pub const fn new() -> OnceBox<T> {
            OnceBox(OnceLock::new())
        }

        pub fn get(&self) -> Option<&T> {
            self.0.get().map(|b| &**b)
        }

        pub fn set(&self, value: Box<T>) -> Result<(), Box<T>> {
            self.0.set(value)
        }

        pub fn get_or_init<F: FnOnce() -> Box<T>>(&self, init: F) -> &T {
            self.0.get_or_init(init)
        }
    }

    impl<T: core::fmt::Debug> core::fmt::Debug for OnceBox<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_tuple("OnceBox").field(&self.get()).finish()
        }
    }
}
