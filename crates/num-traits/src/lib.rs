//! Kinogaki's own numeric traits, published under the crates.io name
//! `num-traits` so the frozen naga and our `half` resolve to it through
//! `[patch.crates-io]`. It is not upstream num-traits: it holds only the traits
//! and methods those two crates call, implemented for the primitive types with
//! std's float functions.
//!
//! The conversions follow num-traits' documented rules, which naga's constant
//! evaluator depends on:
//! - float to integer truncates toward zero and is `None` when the truncated
//!   value does not fit (or the float is NaN or infinite);
//! - integer to float rounds to nearest, ties to even (Rust's `as`);
//! - float to float is Rust's `as` (finite values may overflow to infinity).

pub mod bounds;
pub mod cast;
pub mod float;
pub mod identities;
pub mod int;
pub mod ops;
pub mod real;
pub mod sign;

pub use bounds::Bounded;
pub use cast::{AsPrimitive, FromPrimitive, NumCast, ToPrimitive, cast};
pub use float::{Float, FloatConst, FloatCore};
pub use identities::{One, Zero};
pub use int::PrimInt;
pub use ops::checked::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub};
pub use ops::wrapping::{WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
pub use sign::Signed;

use core::ops::{Div, Rem, Sub};

/// A number: zero, one, equality and the four operations with remainder.
pub trait Num:
    PartialEq + Zero + One + Sub<Output = Self> + Div<Output = Self> + Rem<Output = Self>
{
    type FromStrRadixErr;
    fn from_str_radix(text: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr>;
}

macro_rules! int_num {
    ($($t:ty)*) => {$(
        impl Zero for $t {
            #[inline]
            fn zero() -> $t { 0 }
            #[inline]
            fn is_zero(&self) -> bool { *self == 0 }
        }
        impl One for $t {
            #[inline]
            fn one() -> $t { 1 }
        }
        impl Num for $t {
            type FromStrRadixErr = core::num::ParseIntError;
            fn from_str_radix(text: &str, radix: u32) -> Result<$t, core::num::ParseIntError> {
                <$t>::from_str_radix(text, radix)
            }
        }
        impl Bounded for $t {
            #[inline]
            fn min_value() -> $t { <$t>::MIN }
            #[inline]
            fn max_value() -> $t { <$t>::MAX }
        }
    )*};
}
int_num!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

macro_rules! signed_int {
    ($($t:ty)*) => {$(
        impl Signed for $t {
            #[inline]
            fn abs(&self) -> $t { <$t>::abs(*self) }
            #[inline]
            fn signum(&self) -> $t { <$t>::signum(*self) }
            #[inline]
            fn is_positive(&self) -> bool { *self > 0 }
            #[inline]
            fn is_negative(&self) -> bool { *self < 0 }
        }
    )*};
}
signed_int!(i8 i16 i32 i64 i128 isize);

macro_rules! float_num {
    ($($t:ty)*) => {$(
        impl Zero for $t {
            #[inline]
            fn zero() -> $t { 0.0 }
            #[inline]
            fn is_zero(&self) -> bool { *self == 0.0 }
        }
        impl One for $t {
            #[inline]
            fn one() -> $t { 1.0 }
        }
        impl Num for $t {
            type FromStrRadixErr = core::num::ParseFloatError;
            /// Decimal only (radix 10); naga never parses another radix.
            fn from_str_radix(text: &str, radix: u32) -> Result<$t, core::num::ParseFloatError> {
                assert_eq!(radix, 10, "num-traits (Kinogaki): only radix 10 floats are parsed");
                text.parse()
            }
        }
        impl Bounded for $t {
            #[inline]
            fn min_value() -> $t { <$t>::MIN }
            #[inline]
            fn max_value() -> $t { <$t>::MAX }
        }
        impl Signed for $t {
            #[inline]
            fn abs(&self) -> $t { <$t>::abs(*self) }
            #[inline]
            fn signum(&self) -> $t { <$t>::signum(*self) }
            #[inline]
            fn is_positive(&self) -> bool { *self > 0.0 }
            #[inline]
            fn is_negative(&self) -> bool { *self < 0.0 }
        }
    )*};
}
float_num!(f32 f64);
