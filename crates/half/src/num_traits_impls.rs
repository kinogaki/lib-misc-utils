//! Our num-traits for [`f16`]. Conversions and functions go through f32
//! (exact for every f16) and round back once, as half does.

use crate::f16;
use core::cmp::Ordering;
use core::num::FpCategory;
use num_traits::{
    AsPrimitive, Bounded, Float, FloatConst, FloatCore, FromPrimitive, Num, NumCast, One,
    ToPrimitive, Zero,
};

impl Zero for f16 {
    #[inline]
    fn zero() -> f16 {
        f16::ZERO
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == f16::ZERO
    }
}

impl One for f16 {
    #[inline]
    fn one() -> f16 {
        f16::ONE
    }
}

impl Num for f16 {
    type FromStrRadixErr = core::num::ParseFloatError;
    fn from_str_radix(text: &str, radix: u32) -> Result<f16, core::num::ParseFloatError> {
        <f32 as Num>::from_str_radix(text, radix).map(f16::from_f32)
    }
}

impl Bounded for f16 {
    #[inline]
    fn min_value() -> f16 {
        f16::MIN
    }
    #[inline]
    fn max_value() -> f16 {
        f16::MAX
    }
}

impl ToPrimitive for f16 {
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        f16::to_f32(*self).to_isize()
    }
    #[inline]
    fn to_i8(&self) -> Option<i8> {
        f16::to_f32(*self).to_i8()
    }
    #[inline]
    fn to_i16(&self) -> Option<i16> {
        f16::to_f32(*self).to_i16()
    }
    #[inline]
    fn to_i32(&self) -> Option<i32> {
        f16::to_f32(*self).to_i32()
    }
    #[inline]
    fn to_i64(&self) -> Option<i64> {
        f16::to_f32(*self).to_i64()
    }
    #[inline]
    fn to_i128(&self) -> Option<i128> {
        f16::to_f32(*self).to_i128()
    }
    #[inline]
    fn to_usize(&self) -> Option<usize> {
        f16::to_f32(*self).to_usize()
    }
    #[inline]
    fn to_u8(&self) -> Option<u8> {
        f16::to_f32(*self).to_u8()
    }
    #[inline]
    fn to_u16(&self) -> Option<u16> {
        f16::to_f32(*self).to_u16()
    }
    #[inline]
    fn to_u32(&self) -> Option<u32> {
        f16::to_f32(*self).to_u32()
    }
    #[inline]
    fn to_u64(&self) -> Option<u64> {
        f16::to_f32(*self).to_u64()
    }
    #[inline]
    fn to_u128(&self) -> Option<u128> {
        f16::to_f32(*self).to_u128()
    }
    #[inline]
    fn to_f32(&self) -> Option<f32> {
        Some(f16::to_f32(*self))
    }
    #[inline]
    fn to_f64(&self) -> Option<f64> {
        Some(f16::to_f64(*self))
    }
}

// Integers convert to f32 first (as half does), then round to f16.
impl FromPrimitive for f16 {
    #[inline]
    fn from_isize(n: isize) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_i8(n: i8) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_i16(n: i16) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_i32(n: i32) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_i64(n: i64) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_i128(n: i128) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_usize(n: usize) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_u8(n: u8) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_u16(n: u16) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_u32(n: u32) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_u64(n: u64) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_u128(n: u128) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
    #[inline]
    fn from_f32(n: f32) -> Option<f16> {
        Some(f16::from_f32(n))
    }
    #[inline]
    fn from_f64(n: f64) -> Option<f16> {
        Some(f16::from_f64(n))
    }
}

impl NumCast for f16 {
    #[inline]
    fn from<T: ToPrimitive>(n: T) -> Option<f16> {
        n.to_f32().map(f16::from_f32)
    }
}

macro_rules! as_f16 {
    ($($t:ty)*) => {$(
        impl AsPrimitive<f16> for $t {
            #[inline]
            fn as_(self) -> f16 {
                f16::from_f32(self as f32)
            }
        }
        impl AsPrimitive<$t> for f16 {
            #[inline]
            fn as_(self) -> $t {
                self.to_f32() as $t
            }
        }
    )*};
}
as_f16!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32);

impl AsPrimitive<f16> for f64 {
    #[inline]
    fn as_(self) -> f16 {
        f16::from_f64(self)
    }
}

impl AsPrimitive<f64> for f16 {
    #[inline]
    fn as_(self) -> f64 {
        self.to_f64()
    }
}

impl AsPrimitive<f16> for f16 {
    #[inline]
    fn as_(self) -> f16 {
        self
    }
}

/// The f32 function of `self`, rounded back to f16.
macro_rules! via_f32 {
    ($($name:ident($($arg:ident),*);)*) => {$(
        #[inline]
        fn $name(self, $($arg: f16),*) -> f16 {
            f16::from_f32(self.to_f32().$name($($arg.to_f32()),*))
        }
    )*};
}

/// half's FloatCore min and max: ties (±0) give `other` for min and `self`
/// for max; a NaN operand gives the other operand.
fn core_min(a: f16, b: f16) -> f16 {
    match a.partial_cmp(&b) {
        None => {
            if a.is_nan() {
                b
            } else {
                a
            }
        }
        Some(Ordering::Greater | Ordering::Equal) => b,
        Some(Ordering::Less) => a,
    }
}

fn core_max(a: f16, b: f16) -> f16 {
    match a.partial_cmp(&b) {
        None => {
            if a.is_nan() {
                b
            } else {
                a
            }
        }
        Some(Ordering::Greater | Ordering::Equal) => a,
        Some(Ordering::Less) => b,
    }
}

impl FloatCore for f16 {
    #[inline]
    fn infinity() -> f16 {
        f16::INFINITY
    }
    #[inline]
    fn neg_infinity() -> f16 {
        f16::NEG_INFINITY
    }
    #[inline]
    fn nan() -> f16 {
        f16::NAN
    }
    #[inline]
    fn neg_zero() -> f16 {
        f16::NEG_ZERO
    }
    #[inline]
    fn min_value() -> f16 {
        f16::MIN
    }
    #[inline]
    fn min_positive_value() -> f16 {
        f16::MIN_POSITIVE
    }
    #[inline]
    fn epsilon() -> f16 {
        f16::EPSILON
    }
    #[inline]
    fn max_value() -> f16 {
        f16::MAX
    }
    #[inline]
    fn is_nan(self) -> bool {
        f16::is_nan(self)
    }
    #[inline]
    fn is_infinite(self) -> bool {
        f16::is_infinite(self)
    }
    #[inline]
    fn is_finite(self) -> bool {
        f16::is_finite(self)
    }
    #[inline]
    fn is_normal(self) -> bool {
        f16::is_normal(self)
    }
    #[inline]
    fn is_subnormal(self) -> bool {
        f16::is_subnormal(self)
    }
    #[inline]
    fn classify(self) -> FpCategory {
        f16::classify(self)
    }
    via_f32! { floor(); ceil(); round(); trunc(); fract(); recip(); to_degrees(); to_radians(); }
    #[inline]
    fn abs(self) -> f16 {
        f16::abs(self)
    }
    #[inline]
    fn signum(self) -> f16 {
        f16::signum(self)
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        f16::is_sign_positive(self)
    }
    #[inline]
    fn is_sign_negative(self) -> bool {
        f16::is_sign_negative(self)
    }
    #[inline]
    fn min(self, other: f16) -> f16 {
        core_min(self, other)
    }
    #[inline]
    fn max(self, other: f16) -> f16 {
        core_max(self, other)
    }
    #[inline]
    fn clamp(self, min: f16, max: f16) -> f16 {
        f16::clamp(self, min, max)
    }
    #[inline]
    fn powi(self, exp: i32) -> f16 {
        f16::from_f32(self.to_f32().powi(exp))
    }
    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        FloatCore::integer_decode(self.to_f32())
    }
}

impl Float for f16 {
    #[inline]
    fn nan() -> f16 {
        f16::NAN
    }
    #[inline]
    fn infinity() -> f16 {
        f16::INFINITY
    }
    #[inline]
    fn neg_infinity() -> f16 {
        f16::NEG_INFINITY
    }
    #[inline]
    fn neg_zero() -> f16 {
        f16::NEG_ZERO
    }
    #[inline]
    fn min_value() -> f16 {
        f16::MIN
    }
    #[inline]
    fn min_positive_value() -> f16 {
        f16::MIN_POSITIVE
    }
    #[inline]
    fn epsilon() -> f16 {
        f16::EPSILON
    }
    #[inline]
    fn max_value() -> f16 {
        f16::MAX
    }
    #[inline]
    fn is_nan(self) -> bool {
        f16::is_nan(self)
    }
    #[inline]
    fn is_infinite(self) -> bool {
        f16::is_infinite(self)
    }
    #[inline]
    fn is_finite(self) -> bool {
        f16::is_finite(self)
    }
    #[inline]
    fn is_normal(self) -> bool {
        f16::is_normal(self)
    }
    #[inline]
    fn is_subnormal(self) -> bool {
        f16::is_subnormal(self)
    }
    #[inline]
    fn classify(self) -> FpCategory {
        f16::classify(self)
    }
    via_f32! {
        floor(); ceil(); round(); trunc(); fract(); recip(); sqrt(); exp(); exp2(); ln();
        log2(); log10(); to_degrees(); to_radians(); cbrt(); sin(); cos(); tan(); asin();
        acos(); atan(); exp_m1(); ln_1p(); sinh(); cosh(); tanh(); asinh(); acosh(); atanh();
        mul_add(a, b); powf(n); log(base); hypot(other); atan2(other);
    }
    #[inline]
    fn abs(self) -> f16 {
        f16::abs(self)
    }
    #[inline]
    fn signum(self) -> f16 {
        f16::signum(self)
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        f16::is_sign_positive(self)
    }
    #[inline]
    fn is_sign_negative(self) -> bool {
        f16::is_sign_negative(self)
    }
    #[inline]
    fn powi(self, n: i32) -> f16 {
        f16::from_f32(self.to_f32().powi(n))
    }
    #[inline]
    fn max(self, other: f16) -> f16 {
        f16::max(self, other)
    }
    #[inline]
    fn min(self, other: f16) -> f16 {
        f16::min(self, other)
    }
    #[inline]
    fn clamp(self, min: f16, max: f16) -> f16 {
        f16::clamp(self, min, max)
    }
    #[inline]
    fn abs_sub(self, other: f16) -> f16 {
        f16::from_f32(Float::abs_sub(self.to_f32(), other.to_f32()))
    }
    #[inline]
    fn sin_cos(self) -> (f16, f16) {
        let (sin, cos) = self.to_f32().sin_cos();
        (f16::from_f32(sin), f16::from_f32(cos))
    }
    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        FloatCore::integer_decode(self.to_f32())
    }
    #[inline]
    fn copysign(self, sign: f16) -> f16 {
        f16::copysign(self, sign)
    }
}

macro_rules! constants {
    ($($name:ident)*) => {
        #[allow(non_snake_case)]
        impl FloatConst for f16 {
            $(
                #[inline]
                fn $name() -> f16 {
                    f16::from_f64(core::f64::consts::$name)
                }
            )*
        }
    };
}
constants!(
    E FRAC_1_PI FRAC_1_SQRT_2 FRAC_2_PI FRAC_2_SQRT_PI FRAC_PI_2 FRAC_PI_3 FRAC_PI_4
    FRAC_PI_6 FRAC_PI_8 LN_10 LN_2 LOG10_E LOG2_E PI SQRT_2 TAU LOG10_2 LOG2_10
);
