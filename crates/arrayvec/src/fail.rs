//! Panics, out of line and shared by every element type: generic code calls
//! these instead of formatting a message in each instantiation.

/// A push or insert past the capacity.
#[cold]
#[inline(never)]
#[track_caller]
pub(crate) fn capacity_exceeded(what: &'static str) -> ! {
    panic!("{what}: capacity exceeded")
}

/// An index or range past the length.
#[cold]
#[inline(never)]
#[track_caller]
pub(crate) fn out_of_bounds(what: &'static str, index: usize, len: usize) -> ! {
    panic!("{what}: index {index} out of bounds (len {len})")
}
