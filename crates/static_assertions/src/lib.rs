//! Kinogaki's own `static_assertions`, published under the crates.io name so
//! the frozen wgpu, wgpu-hal and wgpu-types resolve to it through
//! `[patch.crates-io]`. It is not upstream static_assertions: it has the two
//! macros those crates use. Both fail to compile when the assertion is false
//! and generate no code.

/// Assert that a type implements every listed trait.
#[macro_export]
macro_rules! assert_impl_all {
    ($type:ty: $($trait:path),+ $(,)?) => {
        const _: fn() = || {
            fn implements<T: ?Sized $(+ $trait)+>() {}
            implements::<$type>();
        };
    };
}

/// Assert that a type implements none of the listed traits.
///
/// Each listed trait adds one more blanket impl of a helper trait for the
/// types that implement it; naming the helper's item is then ambiguous (a
/// compile error) exactly when some listed trait is implemented.
#[macro_export]
macro_rules! assert_not_impl_any {
    ($type:ty: $($trait:path),+ $(,)?) => {
        const _: fn() = || {
            trait AmbiguousIfImpl<Marker> {
                fn item() {}
            }
            impl<T: ?Sized> AmbiguousIfImpl<()> for T {}
            $({
                struct Implemented;
                impl<T: ?Sized + $trait> AmbiguousIfImpl<Implemented> for T {}
                let _ = Implemented;
            })+
            let _ = <$type as AmbiguousIfImpl<_>>::item;
        };
    };
}
