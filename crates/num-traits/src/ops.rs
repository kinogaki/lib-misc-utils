//! Checked and wrapping arithmetic as traits.

pub mod checked {
    /// `checked_add`: `None` on overflow.
    pub trait CheckedAdd: Sized + core::ops::Add<Self, Output = Self> {
        fn checked_add(&self, v: &Self) -> Option<Self>;
    }
    /// `checked_sub`: `None` on overflow.
    pub trait CheckedSub: Sized + core::ops::Sub<Self, Output = Self> {
        fn checked_sub(&self, v: &Self) -> Option<Self>;
    }
    /// `checked_mul`: `None` on overflow.
    pub trait CheckedMul: Sized + core::ops::Mul<Self, Output = Self> {
        fn checked_mul(&self, v: &Self) -> Option<Self>;
    }
    /// `checked_div`: `None` on overflow or division by zero.
    pub trait CheckedDiv: Sized + core::ops::Div<Self, Output = Self> {
        fn checked_div(&self, v: &Self) -> Option<Self>;
    }
    /// `checked_neg`: `None` on overflow.
    pub trait CheckedNeg: Sized {
        fn checked_neg(&self) -> Option<Self>;
    }
}

pub mod wrapping {
    /// `wrapping_add`: modulo the type's range.
    pub trait WrappingAdd: Sized + core::ops::Add<Self, Output = Self> {
        fn wrapping_add(&self, v: &Self) -> Self;
    }
    /// `wrapping_sub`: modulo the type's range.
    pub trait WrappingSub: Sized + core::ops::Sub<Self, Output = Self> {
        fn wrapping_sub(&self, v: &Self) -> Self;
    }
    /// `wrapping_mul`: modulo the type's range.
    pub trait WrappingMul: Sized + core::ops::Mul<Self, Output = Self> {
        fn wrapping_mul(&self, v: &Self) -> Self;
    }
    /// `wrapping_neg`: modulo the type's range.
    pub trait WrappingNeg: Sized {
        fn wrapping_neg(&self) -> Self;
    }
}

use checked::*;
use wrapping::*;

macro_rules! int_ops {
    ($($t:ty)*) => {$(
        impl CheckedAdd for $t { #[inline] fn checked_add(&self, v: &$t) -> Option<$t> { <$t>::checked_add(*self, *v) } }
        impl CheckedSub for $t { #[inline] fn checked_sub(&self, v: &$t) -> Option<$t> { <$t>::checked_sub(*self, *v) } }
        impl CheckedMul for $t { #[inline] fn checked_mul(&self, v: &$t) -> Option<$t> { <$t>::checked_mul(*self, *v) } }
        impl CheckedDiv for $t { #[inline] fn checked_div(&self, v: &$t) -> Option<$t> { <$t>::checked_div(*self, *v) } }
        impl CheckedNeg for $t { #[inline] fn checked_neg(&self) -> Option<$t> { <$t>::checked_neg(*self) } }
        impl WrappingAdd for $t { #[inline] fn wrapping_add(&self, v: &$t) -> $t { <$t>::wrapping_add(*self, *v) } }
        impl WrappingSub for $t { #[inline] fn wrapping_sub(&self, v: &$t) -> $t { <$t>::wrapping_sub(*self, *v) } }
        impl WrappingMul for $t { #[inline] fn wrapping_mul(&self, v: &$t) -> $t { <$t>::wrapping_mul(*self, *v) } }
        impl WrappingNeg for $t { #[inline] fn wrapping_neg(&self) -> $t { <$t>::wrapping_neg(*self) } }
    )*};
}
int_ops!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
