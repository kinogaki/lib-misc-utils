//! Checked conversions between the primitive number types.
//!
//! - integer to integer: `Some` when the value fits;
//! - integer to float, float to float: Rust's `as` (rounds to nearest, ties to
//!   even; finite floats may overflow to infinity);
//! - float to integer: truncates toward zero, `Some` when the truncated value
//!   fits; NaN and infinities are `None`.
//!
//! Every conversion goes through [`Wide`]: the source widened without loss
//! (an `i128`, a `u128` or an `f64`), then narrowed to the target.

/// Checked conversion of `self` to each primitive type.
pub trait ToPrimitive {
    fn to_isize(&self) -> Option<isize>;
    fn to_i8(&self) -> Option<i8>;
    fn to_i16(&self) -> Option<i16>;
    fn to_i32(&self) -> Option<i32>;
    fn to_i64(&self) -> Option<i64>;
    fn to_i128(&self) -> Option<i128>;
    fn to_usize(&self) -> Option<usize>;
    fn to_u8(&self) -> Option<u8>;
    fn to_u16(&self) -> Option<u16>;
    fn to_u32(&self) -> Option<u32>;
    fn to_u64(&self) -> Option<u64>;
    fn to_u128(&self) -> Option<u128>;
    fn to_f32(&self) -> Option<f32>;
    fn to_f64(&self) -> Option<f64>;
}

/// Checked conversion from each primitive type.
pub trait FromPrimitive: Sized {
    fn from_isize(n: isize) -> Option<Self>;
    fn from_i8(n: i8) -> Option<Self>;
    fn from_i16(n: i16) -> Option<Self>;
    fn from_i32(n: i32) -> Option<Self>;
    fn from_i64(n: i64) -> Option<Self>;
    fn from_i128(n: i128) -> Option<Self>;
    fn from_usize(n: usize) -> Option<Self>;
    fn from_u8(n: u8) -> Option<Self>;
    fn from_u16(n: u16) -> Option<Self>;
    fn from_u32(n: u32) -> Option<Self>;
    fn from_u64(n: u64) -> Option<Self>;
    fn from_u128(n: u128) -> Option<Self>;
    fn from_f32(n: f32) -> Option<Self>;
    fn from_f64(n: f64) -> Option<Self>;
}

/// Checked conversion from any [`ToPrimitive`] value.
pub trait NumCast: Sized + ToPrimitive {
    fn from<T: ToPrimitive>(n: T) -> Option<Self>;
}

/// `n` converted to `U` with [`NumCast`]'s rules.
#[inline]
pub fn cast<T: NumCast, U: NumCast>(n: T) -> Option<U> {
    U::from(n)
}

/// Unchecked `as` conversion between primitive types.
pub trait AsPrimitive<T: 'static + Copy>: 'static + Copy {
    fn as_(self) -> T;
}

/// A primitive value widened without loss.
#[derive(Clone, Copy)]
enum Wide {
    Signed(i128),
    Unsigned(u128),
    Float(f64),
}

trait Primitive: Copy {
    fn wide(self) -> Wide;
    fn narrow(value: Wide) -> Option<Self>;
}

/// Float to integer: the truncated value, when it lies in `[lower, upper)`
/// (the target's `MIN` and `MAX + 1`, powers of two and exact in f64). Both
/// comparisons are false for NaN.
#[inline]
fn truncated_fits(value: f64, lower: f64, upper: f64) -> bool {
    let truncated = value.trunc();
    truncated >= lower && truncated < upper
}

macro_rules! integer {
    ($($t:ty: $wide:ident;)*) => {$(
        impl Primitive for $t {
            #[inline]
            fn wide(self) -> Wide {
                Wide::$wide(self as _)
            }
            #[inline]
            fn narrow(value: Wide) -> Option<$t> {
                match value {
                    Wide::Signed(v) => <$t>::try_from(v).ok(),
                    Wide::Unsigned(v) => <$t>::try_from(v).ok(),
                    Wide::Float(v) => {
                        let lower = <$t>::MIN as f64;
                        // MAX + 1: 2^(BITS - 1) for signed types, 2^BITS for unsigned.
                        let upper = 2f64.powi((<$t>::BITS - (<$t>::MIN != 0) as u32) as i32);
                        truncated_fits(v, lower, upper).then(|| v as $t)
                    }
                }
            }
        }
    )*};
}
integer! {
    i8: Signed; i16: Signed; i32: Signed; i64: Signed; i128: Signed; isize: Signed;
    u8: Unsigned; u16: Unsigned; u32: Unsigned; u64: Unsigned; u128: Unsigned; usize: Unsigned;
}

macro_rules! float {
    ($($t:ty)*) => {$(
        impl Primitive for $t {
            #[inline]
            fn wide(self) -> Wide {
                Wide::Float(self as f64)
            }
            #[inline]
            fn narrow(value: Wide) -> Option<$t> {
                // `as` rounds once in each case: i128/u128 to float directly,
                // and f64 to f32 (an f32 source widened to f64 comes back exact).
                Some(match value {
                    Wide::Signed(v) => v as $t,
                    Wide::Unsigned(v) => v as $t,
                    Wide::Float(v) => v as $t,
                })
            }
        }
    )*};
}
float!(f32 f64);

macro_rules! conversions {
    ($($t:ty)*) => {$(
        impl ToPrimitive for $t {
            #[inline] fn to_isize(&self) -> Option<isize> { Primitive::narrow(self.wide()) }
            #[inline] fn to_i8(&self) -> Option<i8> { Primitive::narrow(self.wide()) }
            #[inline] fn to_i16(&self) -> Option<i16> { Primitive::narrow(self.wide()) }
            #[inline] fn to_i32(&self) -> Option<i32> { Primitive::narrow(self.wide()) }
            #[inline] fn to_i64(&self) -> Option<i64> { Primitive::narrow(self.wide()) }
            #[inline] fn to_i128(&self) -> Option<i128> { Primitive::narrow(self.wide()) }
            #[inline] fn to_usize(&self) -> Option<usize> { Primitive::narrow(self.wide()) }
            #[inline] fn to_u8(&self) -> Option<u8> { Primitive::narrow(self.wide()) }
            #[inline] fn to_u16(&self) -> Option<u16> { Primitive::narrow(self.wide()) }
            #[inline] fn to_u32(&self) -> Option<u32> { Primitive::narrow(self.wide()) }
            #[inline] fn to_u64(&self) -> Option<u64> { Primitive::narrow(self.wide()) }
            #[inline] fn to_u128(&self) -> Option<u128> { Primitive::narrow(self.wide()) }
            #[inline] fn to_f32(&self) -> Option<f32> { Primitive::narrow(self.wide()) }
            #[inline] fn to_f64(&self) -> Option<f64> { Primitive::narrow(self.wide()) }
        }
        impl FromPrimitive for $t {
            #[inline] fn from_isize(n: isize) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_i8(n: i8) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_i16(n: i16) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_i32(n: i32) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_i64(n: i64) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_i128(n: i128) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_usize(n: usize) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_u8(n: u8) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_u16(n: u16) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_u32(n: u32) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_u64(n: u64) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_u128(n: u128) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_f32(n: f32) -> Option<$t> { Primitive::narrow(n.wide()) }
            #[inline] fn from_f64(n: f64) -> Option<$t> { Primitive::narrow(n.wide()) }
        }
    )*};
}
conversions!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64);

macro_rules! num_cast {
    ($($t:ty: $to:ident;)*) => {$(
        impl NumCast for $t {
            #[inline]
            fn from<T: ToPrimitive>(n: T) -> Option<$t> {
                n.$to()
            }
        }
    )*};
}
num_cast! {
    i8: to_i8; i16: to_i16; i32: to_i32; i64: to_i64; i128: to_i128; isize: to_isize;
    u8: to_u8; u16: to_u16; u32: to_u32; u64: to_u64; u128: to_u128; usize: to_usize;
    f32: to_f32; f64: to_f64;
}

macro_rules! as_primitive {
    ($s:ty => $($d:ty)*) => {$(
        impl AsPrimitive<$d> for $s {
            #[inline]
            fn as_(self) -> $d {
                self as $d
            }
        }
    )*};
}
macro_rules! as_primitive_all {
    ($($s:ty)*) => {$(
        as_primitive!($s => i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64);
    )*};
}
as_primitive_all!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64);
