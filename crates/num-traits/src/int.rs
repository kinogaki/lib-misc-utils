//! Primitive integers.

use crate::{Bounded, Num, NumCast};
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

/// The primitive integer types.
pub trait PrimInt:
    Sized
    + Copy
    + Num
    + NumCast
    + Bounded
    + PartialOrd
    + Ord
    + Eq
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
{
    fn count_ones(self) -> u32;
    fn count_zeros(self) -> u32;
    fn leading_zeros(self) -> u32;
    fn trailing_zeros(self) -> u32;
    fn pow(self, exp: u32) -> Self;
}

macro_rules! prim_int {
    ($($t:ty)*) => {$(
        impl PrimInt for $t {
            #[inline] fn count_ones(self) -> u32 { <$t>::count_ones(self) }
            #[inline] fn count_zeros(self) -> u32 { <$t>::count_zeros(self) }
            #[inline] fn leading_zeros(self) -> u32 { <$t>::leading_zeros(self) }
            #[inline] fn trailing_zeros(self) -> u32 { <$t>::trailing_zeros(self) }
            #[inline] fn pow(self, exp: u32) -> $t { <$t>::pow(self, exp) }
        }
    )*};
}
prim_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
