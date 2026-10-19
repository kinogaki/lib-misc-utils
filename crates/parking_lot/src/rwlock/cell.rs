//! The RwLock's data cell: the crate's only unsafe code.
//!
//! `DataCell` hands out borrows of the data only against a guard, and the
//! lock state in `super` guarantees what each guard means:
//!
//! - a read guard exists only while the state counts it as a reader, and no
//!   writer holds the lock then, so shared borrows never overlap a mutable one;
//! - a write guard exists only while the state's write bit is set, which
//!   excludes every reader and other writer, so its mutable borrow is unique.
//!   The borrow is tied to `&mut` of the guard, so one guard cannot hand out
//!   two overlapping mutable borrows.
//!
//! The lock is `Sync` when `T: Send + Sync` (readers on many threads share
//! `&T`; a writer on any thread gets `&mut T`), as std's `RwLock` is.
//! `force_unlock_read` is unsafe for the caller, as in parking_lot: it must
//! hold a read whose guard was forgotten.
#![allow(unsafe_code)]

use super::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use core::cell::UnsafeCell;

pub(super) struct DataCell<T: ?Sized>(UnsafeCell<T>);

impl<T> DataCell<T> {
    pub(super) const fn new(value: T) -> DataCell<T> {
        DataCell(UnsafeCell::new(value))
    }

    pub(super) fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

impl<T: ?Sized> DataCell<T> {
    pub(super) fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }

    pub(super) fn shared<'g>(&'g self, _guard: &'g RwLockReadGuard<'_, T>) -> &'g T {
        // SAFETY: the read guard proves a counted reader and no writer (module docs).
        unsafe { &*self.0.get() }
    }

    pub(super) fn exclusive_ref<'g>(&'g self, _guard: &'g RwLockWriteGuard<'_, T>) -> &'g T {
        // SAFETY: the write guard proves the write bit; the shared borrow is
        // tied to a shared borrow of the guard, so it cannot meet `exclusive`.
        unsafe { &*self.0.get() }
    }

    pub(super) fn exclusive<'g>(&'g self, _guard: &'g mut RwLockWriteGuard<'_, T>) -> &'g mut T {
        // SAFETY: the write guard proves the write bit and is borrowed
        // mutably for as long as the result lives (module docs).
        unsafe { &mut *self.0.get() }
    }
}

// SAFETY: see the module docs; the same bounds as std's RwLock.
unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
// SAFETY: see the module docs; the same bounds as std's RwLock.
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

impl<T: ?Sized> RwLock<T> {
    /// Release a read whose guard was forgotten (`mem::forget`).
    ///
    /// # Safety
    ///
    /// The caller holds a read of this lock that no live guard represents.
    pub unsafe fn force_unlock_read(&self) {
        self.unlock_read();
    }
}
