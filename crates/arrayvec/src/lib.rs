//! Kinogaki's own `arrayvec`, published under the crates.io name so the frozen
//! naga, wgpu, wgpu-core, wgpu-hal and blake3 resolve to it through
//! `[patch.crates-io]`. It is not upstream arrayvec: it has the API those
//! crates call.
//!
//! - [`ArrayVec<T, CAP>`]: a vector with inline storage for at most `CAP`
//!   elements. Its storage is [`buffer::Buffer`], the crate's only unsafe
//!   code: a `[MaybeUninit<T>; CAP]` whose first `len` slots are initialised.
//!   Every other operation (insert, remove, drain, retain, extend...) is safe
//!   code over the buffer's push, pop, truncate and slice views.
//! - [`ArrayString<CAP>`]: a string in `CAP` inline bytes, in safe code.

mod array_string;
mod array_vec;
mod buffer;

pub use array_string::ArrayString;
pub use array_vec::{ArrayVec, Drain, IntoIter};

/// The error of an operation that needs more capacity; it carries the
/// element that did not fit.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapacityError<T = ()> {
    element: T,
}

impl<T> CapacityError<T> {
    pub const fn new(element: T) -> CapacityError<T> {
        CapacityError { element }
    }

    pub fn element(self) -> T {
        self.element
    }

    pub fn simplify(self) -> CapacityError {
        CapacityError { element: () }
    }
}

impl<T> core::fmt::Debug for CapacityError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CapacityError: insufficient capacity")
    }
}

impl<T> core::fmt::Display for CapacityError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "insufficient capacity")
    }
}

impl<T> core::error::Error for CapacityError<T> {}
