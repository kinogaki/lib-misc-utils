//! The binary16 type.

use crate::convert;
use core::cmp::Ordering;
use core::fmt;
use core::iter::{Product, Sum};
use core::num::FpCategory;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

/// An IEEE 754 binary16 ("half precision") float, stored as its bits.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct f16(u16);

impl f16 {
    pub const ZERO: f16 = f16(0x0000);
    pub const NEG_ZERO: f16 = f16(0x8000);
    pub const ONE: f16 = f16(0x3c00);
    pub const NEG_ONE: f16 = f16(0xbc00);
    pub const MAX: f16 = f16(0x7bff);
    pub const MIN: f16 = f16(0xfbff);
    pub const MIN_POSITIVE: f16 = f16(0x0400);
    pub const MIN_POSITIVE_SUBNORMAL: f16 = f16(0x0001);
    pub const MAX_SUBNORMAL: f16 = f16(0x03ff);
    pub const EPSILON: f16 = f16(0x1400);
    pub const INFINITY: f16 = f16(0x7c00);
    pub const NEG_INFINITY: f16 = f16(0xfc00);
    pub const NAN: f16 = f16(0x7e00);
    pub const RADIX: u32 = 2;
    pub const MANTISSA_DIGITS: u32 = 11;
    pub const DIGITS: u32 = 3;
    pub const MAX_EXP: i32 = 16;
    pub const MIN_EXP: i32 = -13;
    pub const MAX_10_EXP: i32 = 4;
    pub const MIN_10_EXP: i32 = -4;

    #[inline]
    pub const fn from_bits(bits: u16) -> f16 {
        f16(bits)
    }

    #[inline]
    pub const fn to_bits(self) -> u16 {
        self.0
    }

    /// Rounded to nearest, ties to even; out of range gives ±∞.
    #[inline]
    pub const fn from_f32(value: f32) -> f16 {
        f16(convert::f32_to_f16(value))
    }

    /// [`f16::from_f32`] in const contexts.
    #[inline]
    pub const fn from_f32_const(value: f32) -> f16 {
        f16(convert::f32_to_f16(value))
    }

    /// Rounded once to nearest, ties to even; out of range gives ±∞.
    #[inline]
    pub const fn from_f64(value: f64) -> f16 {
        f16(convert::f64_to_f16(value))
    }

    /// [`f16::from_f64`] in const contexts.
    #[inline]
    pub const fn from_f64_const(value: f64) -> f16 {
        f16(convert::f64_to_f16(value))
    }

    /// Exact.
    #[inline]
    pub const fn to_f32(self) -> f32 {
        convert::f16_to_f32(self.0)
    }

    /// [`f16::to_f32`] in const contexts.
    #[inline]
    pub const fn to_f32_const(self) -> f32 {
        convert::f16_to_f32(self.0)
    }

    /// Exact.
    #[inline]
    pub const fn to_f64(self) -> f64 {
        convert::f16_to_f64(self.0)
    }

    /// [`f16::to_f64`] in const contexts.
    #[inline]
    pub const fn to_f64_const(self) -> f64 {
        convert::f16_to_f64(self.0)
    }

    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 2] {
        self.0.to_le_bytes()
    }

    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }

    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; 2] {
        self.0.to_ne_bytes()
    }

    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 2]) -> f16 {
        f16(u16::from_le_bytes(bytes))
    }

    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 2]) -> f16 {
        f16(u16::from_be_bytes(bytes))
    }

    #[inline]
    pub const fn from_ne_bytes(bytes: [u8; 2]) -> f16 {
        f16(u16::from_ne_bytes(bytes))
    }

    #[inline]
    pub const fn is_nan(self) -> bool {
        self.0 & 0x7fff > 0x7c00
    }

    #[inline]
    pub const fn is_infinite(self) -> bool {
        self.0 & 0x7fff == 0x7c00
    }

    #[inline]
    pub const fn is_finite(self) -> bool {
        self.0 & 0x7c00 != 0x7c00
    }

    #[inline]
    pub const fn is_normal(self) -> bool {
        let exponent = self.0 & 0x7c00;
        exponent != 0x7c00 && exponent != 0
    }

    #[inline]
    pub const fn is_subnormal(self) -> bool {
        self.0 & 0x7c00 == 0 && self.0 & 0x03ff != 0
    }

    pub const fn classify(self) -> FpCategory {
        let exponent = self.0 & 0x7c00;
        let mantissa = self.0 & 0x03ff;
        match (exponent, mantissa) {
            (0, 0) => FpCategory::Zero,
            (0, _) => FpCategory::Subnormal,
            (0x7c00, 0) => FpCategory::Infinite,
            (0x7c00, _) => FpCategory::Nan,
            _ => FpCategory::Normal,
        }
    }

    /// NaN for NaN, otherwise ±1 by the sign bit.
    #[inline]
    pub const fn signum(self) -> f16 {
        if self.is_nan() {
            self
        } else if self.0 & 0x8000 != 0 {
            f16::NEG_ONE
        } else {
            f16::ONE
        }
    }

    #[inline]
    pub const fn is_sign_positive(self) -> bool {
        self.0 & 0x8000 == 0
    }

    #[inline]
    pub const fn is_sign_negative(self) -> bool {
        self.0 & 0x8000 != 0
    }

    #[inline]
    pub const fn copysign(self, sign: f16) -> f16 {
        f16((self.0 & 0x7fff) | (sign.0 & 0x8000))
    }

    #[inline]
    pub const fn abs(self) -> f16 {
        f16(self.0 & 0x7fff)
    }

    /// The larger value; if one is NaN, the other.
    #[inline]
    pub fn max(self, other: f16) -> f16 {
        if self.is_nan() || other > self {
            other
        } else {
            self
        }
    }

    /// The smaller value; if one is NaN, the other.
    #[inline]
    pub fn min(self, other: f16) -> f16 {
        if self.is_nan() || other < self {
            other
        } else {
            self
        }
    }

    /// `self` limited to `[min, max]` (NaN stays NaN). Panics unless `min <= max`.
    #[inline]
    pub fn clamp(self, min: f16, max: f16) -> f16 {
        assert!(min <= max);
        let mut x = self;
        if x < min {
            x = min;
        }
        if x > max {
            x = max;
        }
        x
    }

    /// IEEE 754 totalOrder.
    #[inline]
    pub fn total_cmp(&self, other: &f16) -> Ordering {
        let key = |bits: u16| {
            let bits = bits as i16;
            bits ^ ((((bits >> 15) as u16) >> 1) as i16)
        };
        key(self.0).cmp(&key(other.0))
    }
}

impl From<f16> for f32 {
    #[inline]
    fn from(x: f16) -> f32 {
        x.to_f32()
    }
}

impl From<f16> for f64 {
    #[inline]
    fn from(x: f16) -> f64 {
        x.to_f64()
    }
}

impl From<i8> for f16 {
    #[inline]
    fn from(x: i8) -> f16 {
        f16::from_f32(f32::from(x))
    }
}

impl From<u8> for f16 {
    #[inline]
    fn from(x: u8) -> f16 {
        f16::from_f32(f32::from(x))
    }
}

impl PartialEq for f16 {
    #[inline]
    fn eq(&self, other: &f16) -> bool {
        !self.is_nan() && !other.is_nan() && (self.0 == other.0 || (self.0 | other.0) & 0x7fff == 0)
    }
}

impl PartialOrd for f16 {
    #[inline]
    fn partial_cmp(&self, other: &f16) -> Option<Ordering> {
        // Exact in f32, including ±0 and NaN.
        self.to_f32().partial_cmp(&other.to_f32())
    }
}

impl core::str::FromStr for f16 {
    type Err = core::num::ParseFloatError;
    /// Parsed as f32, then rounded to f16.
    fn from_str(text: &str) -> Result<f16, core::num::ParseFloatError> {
        text.parse::<f32>().map(f16::from_f32)
    }
}

// Text is the f32 value's text: every f16 is an f32, and f32's shortest
// round-trip spelling is what naga writes into shader source.
impl fmt::Debug for f16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_f32(), f)
    }
}

impl fmt::Display for f16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_f32(), f)
    }
}

impl fmt::LowerExp for f16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerExp::fmt(&self.to_f32(), f)
    }
}

impl fmt::UpperExp for f16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperExp::fmt(&self.to_f32(), f)
    }
}

impl Neg for f16 {
    type Output = f16;
    #[inline]
    fn neg(self) -> f16 {
        f16(self.0 ^ 0x8000)
    }
}

impl Neg for &f16 {
    type Output = f16;
    #[inline]
    fn neg(self) -> f16 {
        -*self
    }
}

macro_rules! binary_op {
    ($($trait:ident $method:ident $assign_trait:ident $assign:ident $op:tt;)*) => {$(
        impl $trait for f16 {
            type Output = f16;
            #[inline]
            fn $method(self, rhs: f16) -> f16 {
                f16::from_f32(self.to_f32() $op rhs.to_f32())
            }
        }
        impl $trait<&f16> for f16 {
            type Output = f16;
            #[inline]
            fn $method(self, rhs: &f16) -> f16 {
                self $op *rhs
            }
        }
        impl $trait<f16> for &f16 {
            type Output = f16;
            #[inline]
            fn $method(self, rhs: f16) -> f16 {
                *self $op rhs
            }
        }
        impl $trait<&f16> for &f16 {
            type Output = f16;
            #[inline]
            fn $method(self, rhs: &f16) -> f16 {
                *self $op *rhs
            }
        }
        impl $assign_trait for f16 {
            #[inline]
            fn $assign(&mut self, rhs: f16) {
                *self = *self $op rhs;
            }
        }
        impl $assign_trait<&f16> for f16 {
            #[inline]
            fn $assign(&mut self, rhs: &f16) {
                *self = *self $op *rhs;
            }
        }
    )*};
}
binary_op! {
    Add add AddAssign add_assign +;
    Sub sub SubAssign sub_assign -;
    Mul mul MulAssign mul_assign *;
    Div div DivAssign div_assign /;
    Rem rem RemAssign rem_assign %;
}

// Sums and products round after every step, as binary16 hardware does.
impl Sum for f16 {
    fn sum<I: Iterator<Item = f16>>(iter: I) -> f16 {
        iter.fold(f16::ZERO, |acc, x| acc + x)
    }
}

impl<'a> Sum<&'a f16> for f16 {
    fn sum<I: Iterator<Item = &'a f16>>(iter: I) -> f16 {
        iter.fold(f16::ZERO, |acc, x| acc + *x)
    }
}

impl Product for f16 {
    fn product<I: Iterator<Item = f16>>(iter: I) -> f16 {
        iter.fold(f16::ONE, |acc, x| acc * x)
    }
}

impl<'a> Product<&'a f16> for f16 {
    fn product<I: Iterator<Item = &'a f16>>(iter: I) -> f16 {
        iter.fold(f16::ONE, |acc, x| acc * *x)
    }
}
