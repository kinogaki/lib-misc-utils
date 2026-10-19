//! Kinogaki's own `smallvec`, published under the crates.io name so the
//! frozen wgpu, wgpu-core, wgpu-hal and harfrust resolve to it through
//! `[patch.crates-io]`, and used by kinogaki-ui. It is not upstream smallvec:
//! it has the API those crates call.
//!
//! [`SmallVec<[T; N]>`] keeps up to `N` elements inline (in our `arrayvec`)
//! and moves them to a `Vec` when an element no longer fits ("spills"). As
//! upstream, it stays on the heap after that until `shrink_to_fit` moves
//! elements that fit back inline. Safe code throughout; the only unsafe code
//! underneath is arrayvec's documented inline buffer.

mod small_vec;

pub use small_vec::{Drain, IntoIter, SmallVec};

/// An array type `[T; N]` that names a small vector's element type and
/// inline capacity.
pub trait Array {
    type Item;
    #[doc(hidden)]
    type Inline: Default + small_vec::Inline<Self::Item>;
    /// The inline capacity.
    fn size() -> usize;
}

impl<T, const N: usize> Array for [T; N] {
    type Item = T;
    type Inline = arrayvec::ArrayVec<T, N>;
    #[inline]
    fn size() -> usize {
        N
    }
}

/// A [`SmallVec`] of the given elements, or of `n` clones of one element.
#[macro_export]
macro_rules! smallvec {
    ($elem:expr; $n:expr) => {
        $crate::SmallVec::from_elem($elem, $n)
    };
    ($($x:expr),* $(,)?) => {{
        let mut vec = $crate::SmallVec::new();
        $(vec.push($x);)*
        vec
    }};
}
