//! The inline buffer: the crate's only unsafe code.
//!
//! Invariant: `len <= CAP`, and exactly the slots `data[..len]` hold
//! initialised values (owned by the buffer). Each method keeps it:
//!
//! - `push` writes slot `len` (uninitialised by the invariant) before raising `len`;
//! - `pop` lowers `len` before reading slot `len` out, so the value has one owner;
//! - `truncate` lowers `len` first, then drops the slots it gave up (a panic in
//!   a destructor leaves the remaining tail leaked, never dropped twice);
//! - the slice views cover exactly `data[..len]`;
//! - `Drop` drops `data[..len]`.
//!
//! [`Drainer`], the owning iterator, keeps `data[start..end]` initialised:
//! each read moves one slot out after narrowing the range past it, and its
//! `Drop` drops what is left.
#![allow(unsafe_code)]

use core::mem::MaybeUninit;

pub(crate) struct Buffer<T, const CAP: usize> {
    data: [MaybeUninit<T>; CAP],
    len: usize,
}

impl<T, const CAP: usize> Buffer<T, CAP> {
    pub(crate) const fn new() -> Buffer<T, CAP> {
        Buffer {
            data: [const { MaybeUninit::uninit() }; CAP],
            len: 0,
        }
    }

    #[inline]
    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    /// Append `value`, or give it back when full.
    #[inline]
    pub(crate) fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == CAP {
            return Err(value);
        }
        self.data[self.len].write(value);
        self.len += 1;
        Ok(())
    }

    #[inline]
    pub(crate) fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: slot `len` was initialised and is now outside `..len`, so
        // this read takes its only owner.
        Some(unsafe { self.data[self.len].assume_init_read() })
    }

    pub(crate) fn truncate(&mut self, len: usize) {
        while self.len > len {
            drop(self.pop());
        }
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[T] {
        // SAFETY: `data[..len]` is initialised; MaybeUninit<T> has T's layout.
        unsafe { &*(core::ptr::from_ref(&self.data[..self.len]) as *const [T]) }
    }

    #[inline]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: as in `as_slice`, borrowed mutably.
        unsafe { &mut *(core::ptr::from_mut(&mut self.data[..self.len]) as *mut [T]) }
    }
}

impl<T, const CAP: usize> Buffer<T, CAP> {
    /// The elements as an owning iterator (the buffer gives them up).
    pub(crate) fn into_drainer(mut self) -> Drainer<T, CAP> {
        let end = self.len;
        self.len = 0;
        let data = core::mem::replace(&mut self.data, [const { MaybeUninit::uninit() }; CAP]);
        Drainer {
            data,
            start: 0,
            end,
        }
    }
}

/// Owns the values in `data[start..end]` and hands them out from either end.
pub(crate) struct Drainer<T, const CAP: usize> {
    data: [MaybeUninit<T>; CAP],
    start: usize,
    end: usize,
}

impl<T, const CAP: usize> Drainer<T, CAP> {
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub(crate) fn front(&mut self) -> Option<T> {
        if self.start == self.end {
            return None;
        }
        self.start += 1;
        // SAFETY: slot start - 1 was in the owned range and is now outside it.
        Some(unsafe { self.data[self.start - 1].assume_init_read() })
    }

    #[inline]
    pub(crate) fn back(&mut self) -> Option<T> {
        if self.start == self.end {
            return None;
        }
        self.end -= 1;
        // SAFETY: slot end was in the owned range and is now outside it.
        Some(unsafe { self.data[self.end].assume_init_read() })
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        // SAFETY: `data[start..end]` is initialised; MaybeUninit<T> has T's layout.
        unsafe { &*(core::ptr::from_ref(&self.data[self.start..self.end]) as *const [T]) }
    }
}

impl<T, const CAP: usize> Drop for Drainer<T, CAP> {
    fn drop(&mut self) {
        // SAFETY: `data[start..end]` is initialised and owned; dropped once here.
        unsafe {
            core::ptr::drop_in_place(
                core::ptr::from_mut(&mut self.data[self.start..self.end]) as *mut [T]
            )
        }
    }
}

impl<T, const CAP: usize> Drop for Buffer<T, CAP> {
    fn drop(&mut self) {
        // SAFETY: `data[..len]` is initialised and owned; dropped once here.
        unsafe { core::ptr::drop_in_place(self.as_mut_slice()) }
    }
}
