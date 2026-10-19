//! A mutual exclusion lock without poisoning, on std's `Mutex`.

use core::fmt;
use core::ops::{Deref, DerefMut};

/// A mutual exclusion lock. A panic while it is held does not poison it.
#[derive(Default)]
pub struct Mutex<T: ?Sized> {
    inner: std::sync::Mutex<T>,
}

/// `Mutex::new` in const contexts.
pub const fn const_mutex<T>(value: T) -> Mutex<T> {
    Mutex::new(value)
}

impl<T> Mutex<T> {
    #[inline]
    pub const fn new(value: T) -> Mutex<T> {
        Mutex {
            inner: std::sync::Mutex::new(value),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Block until the lock is free, then hold it.
    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        MutexGuard { inner: Some(guard) }
    }

    /// Hold the lock if it is free now.
    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        match self.inner.try_lock() {
            Ok(guard) => Some(MutexGuard { inner: Some(guard) }),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => Some(MutexGuard {
                inner: Some(poisoned.into_inner()),
            }),
            Err(std::sync::TryLockError::WouldBlock) => None,
        }
    }

    /// Whether some thread holds the lock (a snapshot; for assertions).
    #[inline]
    pub fn is_locked(&self) -> bool {
        matches!(
            self.inner.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        )
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Mutex<T> {
        Mutex::new(value)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
            None => f
                .debug_struct("Mutex")
                .field("data", &format_args!("<locked>"))
                .finish(),
        }
    }
}

/// Holds a [`Mutex`] until dropped.
pub struct MutexGuard<'a, T: ?Sized> {
    // `None` only while a `Condvar` wait has handed the std guard to std's
    // condition variable.
    pub(crate) inner: Option<std::sync::MutexGuard<'a, T>>,
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        self.inner
            .as_deref()
            .expect("parking_lot: guard in use by a Condvar wait")
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.inner
            .as_deref_mut()
            .expect("parking_lot: guard in use by a Condvar wait")
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
