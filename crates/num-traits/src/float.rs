//! Floating-point traits: [`FloatCore`] (no transcendental functions),
//! [`Float`] (everything std's `f32`/`f64` methods offer), [`Real`] (a
//! `Float` without the special values) and [`FloatConst`].
//!
//! For `f32` and `f64` every method is std's method of the same name, so a
//! shader constant naga folds is what Rust computes for that expression.

use crate::{Num, NumCast};
use core::num::FpCategory;
use core::ops::Neg;

/// Floats without the functions that need a math library.
pub trait FloatCore: Num + NumCast + Neg<Output = Self> + PartialOrd + Copy {
    fn infinity() -> Self;
    fn neg_infinity() -> Self;
    fn nan() -> Self;
    fn neg_zero() -> Self;
    fn min_value() -> Self;
    fn min_positive_value() -> Self;
    fn epsilon() -> Self;
    fn max_value() -> Self;
    fn is_nan(self) -> bool;
    fn is_infinite(self) -> bool;
    fn is_finite(self) -> bool;
    fn is_normal(self) -> bool;
    fn is_subnormal(self) -> bool;
    fn classify(self) -> FpCategory;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn trunc(self) -> Self;
    fn fract(self) -> Self;
    fn abs(self) -> Self;
    fn signum(self) -> Self;
    fn is_sign_positive(self) -> bool;
    fn is_sign_negative(self) -> bool;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn clamp(self, min: Self, max: Self) -> Self;
    fn recip(self) -> Self;
    fn powi(self, exp: i32) -> Self;
    fn to_degrees(self) -> Self;
    fn to_radians(self) -> Self;
    /// `(mantissa, exponent, sign)` with `value = sign * mantissa * 2^exponent`.
    fn integer_decode(self) -> (u64, i16, i8);
}

/// Floats with the full set of std's float methods.
pub trait Float: Num + Copy + NumCast + PartialOrd + Neg<Output = Self> {
    fn nan() -> Self;
    fn infinity() -> Self;
    fn neg_infinity() -> Self;
    fn neg_zero() -> Self;
    fn min_value() -> Self;
    fn min_positive_value() -> Self;
    fn epsilon() -> Self;
    fn max_value() -> Self;
    fn is_nan(self) -> bool;
    fn is_infinite(self) -> bool;
    fn is_finite(self) -> bool;
    fn is_normal(self) -> bool;
    fn is_subnormal(self) -> bool;
    fn classify(self) -> FpCategory;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn trunc(self) -> Self;
    fn fract(self) -> Self;
    fn abs(self) -> Self;
    fn signum(self) -> Self;
    fn is_sign_positive(self) -> bool;
    fn is_sign_negative(self) -> bool;
    fn mul_add(self, a: Self, b: Self) -> Self;
    fn recip(self) -> Self;
    fn powi(self, n: i32) -> Self;
    fn powf(self, n: Self) -> Self;
    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn exp2(self) -> Self;
    fn ln(self) -> Self;
    fn log(self, base: Self) -> Self;
    fn log2(self) -> Self;
    fn log10(self) -> Self;
    fn to_degrees(self) -> Self;
    fn to_radians(self) -> Self;
    fn max(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn clamp(self, min: Self, max: Self) -> Self;
    /// `max(self - other, 0)` (NaN when either is NaN).
    fn abs_sub(self, other: Self) -> Self;
    fn cbrt(self) -> Self;
    fn hypot(self, other: Self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
    fn atan2(self, other: Self) -> Self;
    fn sin_cos(self) -> (Self, Self);
    fn exp_m1(self) -> Self;
    fn ln_1p(self) -> Self;
    fn sinh(self) -> Self;
    fn cosh(self) -> Self;
    fn tanh(self) -> Self;
    fn asinh(self) -> Self;
    fn acosh(self) -> Self;
    fn atanh(self) -> Self;
    /// `(mantissa, exponent, sign)` with `value = sign * mantissa * 2^exponent`.
    fn integer_decode(self) -> (u64, i16, i8);
    fn copysign(self, sign: Self) -> Self;
}

/// Real numbers: [`Float`] without NaN, the infinities and classification.
/// Every `Float` is one.
pub trait Real: Num + Copy + NumCast + PartialOrd + Neg<Output = Self> {
    fn min_value() -> Self;
    fn min_positive_value() -> Self;
    fn epsilon() -> Self;
    fn max_value() -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn trunc(self) -> Self;
    fn fract(self) -> Self;
    fn abs(self) -> Self;
    fn signum(self) -> Self;
    fn is_sign_positive(self) -> bool;
    fn is_sign_negative(self) -> bool;
    fn mul_add(self, a: Self, b: Self) -> Self;
    fn recip(self) -> Self;
    fn powi(self, n: i32) -> Self;
    fn powf(self, n: Self) -> Self;
    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn exp2(self) -> Self;
    fn ln(self) -> Self;
    fn log(self, base: Self) -> Self;
    fn log2(self) -> Self;
    fn log10(self) -> Self;
    fn to_degrees(self) -> Self;
    fn to_radians(self) -> Self;
    fn max(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn abs_sub(self, other: Self) -> Self;
    fn cbrt(self) -> Self;
    fn hypot(self, other: Self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
    fn atan2(self, other: Self) -> Self;
    fn sin_cos(self) -> (Self, Self);
    fn exp_m1(self) -> Self;
    fn ln_1p(self) -> Self;
    fn sinh(self) -> Self;
    fn cosh(self) -> Self;
    fn tanh(self) -> Self;
    fn asinh(self) -> Self;
    fn acosh(self) -> Self;
    fn atanh(self) -> Self;
}

macro_rules! real_from_float {
    (statics: $($s:ident)*; methods: $($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        impl<T: Float> Real for T {
            $(
                #[inline]
                fn $s() -> T {
                    <T as Float>::$s()
                }
            )*
            $(
                #[inline]
                fn $name(self, $($arg: $ty),*) -> $ret {
                    <T as Float>::$name(self, $($arg),*)
                }
            )*
        }
    };
}
real_from_float! {
    statics: min_value min_positive_value epsilon max_value;
    methods:
    floor() -> T; ceil() -> T; round() -> T; trunc() -> T; fract() -> T;
    abs() -> T; signum() -> T; is_sign_positive() -> bool; is_sign_negative() -> bool;
    mul_add(a: T, b: T) -> T; recip() -> T; powi(n: i32) -> T; powf(n: T) -> T;
    sqrt() -> T; exp() -> T; exp2() -> T; ln() -> T; log(base: T) -> T;
    log2() -> T; log10() -> T; to_degrees() -> T; to_radians() -> T;
    max(o: T) -> T; min(o: T) -> T; abs_sub(o: T) -> T; cbrt() -> T;
    hypot(o: T) -> T; sin() -> T; cos() -> T; tan() -> T; asin() -> T;
    acos() -> T; atan() -> T; atan2(o: T) -> T; sin_cos() -> (T, T);
    exp_m1() -> T; ln_1p() -> T; sinh() -> T; cosh() -> T; tanh() -> T;
    asinh() -> T; acosh() -> T; atanh() -> T;
}

/// Mathematical constants as functions of the float type.
#[allow(non_snake_case)]
pub trait FloatConst {
    fn E() -> Self;
    fn FRAC_1_PI() -> Self;
    fn FRAC_1_SQRT_2() -> Self;
    fn FRAC_2_PI() -> Self;
    fn FRAC_2_SQRT_PI() -> Self;
    fn FRAC_PI_2() -> Self;
    fn FRAC_PI_3() -> Self;
    fn FRAC_PI_4() -> Self;
    fn FRAC_PI_6() -> Self;
    fn FRAC_PI_8() -> Self;
    fn LN_10() -> Self;
    fn LN_2() -> Self;
    fn LOG10_E() -> Self;
    fn LOG2_E() -> Self;
    fn PI() -> Self;
    fn SQRT_2() -> Self;
    fn TAU() -> Self;
    fn LOG10_2() -> Self;
    fn LOG2_10() -> Self;
}

macro_rules! float_impls {
    ($t:ident, $exponent_bits:expr, $mantissa_bits:expr) => {
        impl FloatCore for $t {
            #[inline]
            fn infinity() -> $t {
                $t::INFINITY
            }
            #[inline]
            fn neg_infinity() -> $t {
                $t::NEG_INFINITY
            }
            #[inline]
            fn nan() -> $t {
                $t::NAN
            }
            #[inline]
            fn neg_zero() -> $t {
                -0.0
            }
            #[inline]
            fn min_value() -> $t {
                $t::MIN
            }
            #[inline]
            fn min_positive_value() -> $t {
                $t::MIN_POSITIVE
            }
            #[inline]
            fn epsilon() -> $t {
                $t::EPSILON
            }
            #[inline]
            fn max_value() -> $t {
                $t::MAX
            }
            #[inline]
            fn is_nan(self) -> bool {
                $t::is_nan(self)
            }
            #[inline]
            fn is_infinite(self) -> bool {
                $t::is_infinite(self)
            }
            #[inline]
            fn is_finite(self) -> bool {
                $t::is_finite(self)
            }
            #[inline]
            fn is_normal(self) -> bool {
                $t::is_normal(self)
            }
            #[inline]
            fn is_subnormal(self) -> bool {
                $t::is_subnormal(self)
            }
            #[inline]
            fn classify(self) -> FpCategory {
                $t::classify(self)
            }
            #[inline]
            fn floor(self) -> $t {
                $t::floor(self)
            }
            #[inline]
            fn ceil(self) -> $t {
                $t::ceil(self)
            }
            #[inline]
            fn round(self) -> $t {
                $t::round(self)
            }
            #[inline]
            fn trunc(self) -> $t {
                $t::trunc(self)
            }
            #[inline]
            fn fract(self) -> $t {
                $t::fract(self)
            }
            #[inline]
            fn abs(self) -> $t {
                $t::abs(self)
            }
            #[inline]
            fn signum(self) -> $t {
                $t::signum(self)
            }
            #[inline]
            fn is_sign_positive(self) -> bool {
                $t::is_sign_positive(self)
            }
            #[inline]
            fn is_sign_negative(self) -> bool {
                $t::is_sign_negative(self)
            }
            #[inline]
            fn min(self, other: $t) -> $t {
                $t::min(self, other)
            }
            #[inline]
            fn max(self, other: $t) -> $t {
                $t::max(self, other)
            }
            #[inline]
            fn clamp(self, min: $t, max: $t) -> $t {
                $t::clamp(self, min, max)
            }
            #[inline]
            fn recip(self) -> $t {
                $t::recip(self)
            }
            #[inline]
            fn powi(self, exp: i32) -> $t {
                $t::powi(self, exp)
            }
            #[inline]
            fn to_degrees(self) -> $t {
                $t::to_degrees(self)
            }
            #[inline]
            fn to_radians(self) -> $t {
                $t::to_radians(self)
            }
            #[inline]
            fn integer_decode(self) -> (u64, i16, i8) {
                integer_decode(self.to_bits().into(), $exponent_bits, $mantissa_bits)
            }
        }
        impl Float for $t {
            #[inline]
            fn nan() -> $t {
                $t::NAN
            }
            #[inline]
            fn infinity() -> $t {
                $t::INFINITY
            }
            #[inline]
            fn neg_infinity() -> $t {
                $t::NEG_INFINITY
            }
            #[inline]
            fn neg_zero() -> $t {
                -0.0
            }
            #[inline]
            fn min_value() -> $t {
                $t::MIN
            }
            #[inline]
            fn min_positive_value() -> $t {
                $t::MIN_POSITIVE
            }
            #[inline]
            fn epsilon() -> $t {
                $t::EPSILON
            }
            #[inline]
            fn max_value() -> $t {
                $t::MAX
            }
            #[inline]
            fn is_nan(self) -> bool {
                $t::is_nan(self)
            }
            #[inline]
            fn is_infinite(self) -> bool {
                $t::is_infinite(self)
            }
            #[inline]
            fn is_finite(self) -> bool {
                $t::is_finite(self)
            }
            #[inline]
            fn is_normal(self) -> bool {
                $t::is_normal(self)
            }
            #[inline]
            fn is_subnormal(self) -> bool {
                $t::is_subnormal(self)
            }
            #[inline]
            fn classify(self) -> FpCategory {
                $t::classify(self)
            }
            #[inline]
            fn floor(self) -> $t {
                $t::floor(self)
            }
            #[inline]
            fn ceil(self) -> $t {
                $t::ceil(self)
            }
            #[inline]
            fn round(self) -> $t {
                $t::round(self)
            }
            #[inline]
            fn trunc(self) -> $t {
                $t::trunc(self)
            }
            #[inline]
            fn fract(self) -> $t {
                $t::fract(self)
            }
            #[inline]
            fn abs(self) -> $t {
                $t::abs(self)
            }
            #[inline]
            fn signum(self) -> $t {
                $t::signum(self)
            }
            #[inline]
            fn is_sign_positive(self) -> bool {
                $t::is_sign_positive(self)
            }
            #[inline]
            fn is_sign_negative(self) -> bool {
                $t::is_sign_negative(self)
            }
            #[inline]
            fn mul_add(self, a: $t, b: $t) -> $t {
                $t::mul_add(self, a, b)
            }
            #[inline]
            fn recip(self) -> $t {
                $t::recip(self)
            }
            #[inline]
            fn powi(self, n: i32) -> $t {
                $t::powi(self, n)
            }
            #[inline]
            fn powf(self, n: $t) -> $t {
                $t::powf(self, n)
            }
            #[inline]
            fn sqrt(self) -> $t {
                $t::sqrt(self)
            }
            #[inline]
            fn exp(self) -> $t {
                $t::exp(self)
            }
            #[inline]
            fn exp2(self) -> $t {
                $t::exp2(self)
            }
            #[inline]
            fn ln(self) -> $t {
                $t::ln(self)
            }
            #[inline]
            fn log(self, base: $t) -> $t {
                $t::log(self, base)
            }
            #[inline]
            fn log2(self) -> $t {
                $t::log2(self)
            }
            #[inline]
            fn log10(self) -> $t {
                $t::log10(self)
            }
            #[inline]
            fn to_degrees(self) -> $t {
                $t::to_degrees(self)
            }
            #[inline]
            fn to_radians(self) -> $t {
                $t::to_radians(self)
            }
            #[inline]
            fn max(self, other: $t) -> $t {
                $t::max(self, other)
            }
            #[inline]
            fn min(self, other: $t) -> $t {
                $t::min(self, other)
            }
            #[inline]
            fn clamp(self, min: $t, max: $t) -> $t {
                $t::clamp(self, min, max)
            }
            #[inline]
            fn abs_sub(self, other: $t) -> $t {
                if self <= other { 0.0 } else { self - other }
            }
            #[inline]
            fn cbrt(self) -> $t {
                $t::cbrt(self)
            }
            #[inline]
            fn hypot(self, other: $t) -> $t {
                $t::hypot(self, other)
            }
            #[inline]
            fn sin(self) -> $t {
                $t::sin(self)
            }
            #[inline]
            fn cos(self) -> $t {
                $t::cos(self)
            }
            #[inline]
            fn tan(self) -> $t {
                $t::tan(self)
            }
            #[inline]
            fn asin(self) -> $t {
                $t::asin(self)
            }
            #[inline]
            fn acos(self) -> $t {
                $t::acos(self)
            }
            #[inline]
            fn atan(self) -> $t {
                $t::atan(self)
            }
            #[inline]
            fn atan2(self, other: $t) -> $t {
                $t::atan2(self, other)
            }
            #[inline]
            fn sin_cos(self) -> ($t, $t) {
                $t::sin_cos(self)
            }
            #[inline]
            fn exp_m1(self) -> $t {
                $t::exp_m1(self)
            }
            #[inline]
            fn ln_1p(self) -> $t {
                $t::ln_1p(self)
            }
            #[inline]
            fn sinh(self) -> $t {
                $t::sinh(self)
            }
            #[inline]
            fn cosh(self) -> $t {
                $t::cosh(self)
            }
            #[inline]
            fn tanh(self) -> $t {
                $t::tanh(self)
            }
            #[inline]
            fn asinh(self) -> $t {
                $t::asinh(self)
            }
            #[inline]
            fn acosh(self) -> $t {
                $t::acosh(self)
            }
            #[inline]
            fn atanh(self) -> $t {
                $t::atanh(self)
            }
            #[inline]
            fn integer_decode(self) -> (u64, i16, i8) {
                FloatCore::integer_decode(self)
            }
            #[inline]
            fn copysign(self, sign: $t) -> $t {
                $t::copysign(self, sign)
            }
        }
        impl FloatConst for $t {
            #[inline]
            fn E() -> $t {
                core::$t::consts::E
            }
            #[inline]
            fn FRAC_1_PI() -> $t {
                core::$t::consts::FRAC_1_PI
            }
            #[inline]
            fn FRAC_1_SQRT_2() -> $t {
                core::$t::consts::FRAC_1_SQRT_2
            }
            #[inline]
            fn FRAC_2_PI() -> $t {
                core::$t::consts::FRAC_2_PI
            }
            #[inline]
            fn FRAC_2_SQRT_PI() -> $t {
                core::$t::consts::FRAC_2_SQRT_PI
            }
            #[inline]
            fn FRAC_PI_2() -> $t {
                core::$t::consts::FRAC_PI_2
            }
            #[inline]
            fn FRAC_PI_3() -> $t {
                core::$t::consts::FRAC_PI_3
            }
            #[inline]
            fn FRAC_PI_4() -> $t {
                core::$t::consts::FRAC_PI_4
            }
            #[inline]
            fn FRAC_PI_6() -> $t {
                core::$t::consts::FRAC_PI_6
            }
            #[inline]
            fn FRAC_PI_8() -> $t {
                core::$t::consts::FRAC_PI_8
            }
            #[inline]
            fn LN_10() -> $t {
                core::$t::consts::LN_10
            }
            #[inline]
            fn LN_2() -> $t {
                core::$t::consts::LN_2
            }
            #[inline]
            fn LOG10_E() -> $t {
                core::$t::consts::LOG10_E
            }
            #[inline]
            fn LOG2_E() -> $t {
                core::$t::consts::LOG2_E
            }
            #[inline]
            fn PI() -> $t {
                core::$t::consts::PI
            }
            #[inline]
            fn SQRT_2() -> $t {
                core::$t::consts::SQRT_2
            }
            #[inline]
            fn TAU() -> $t {
                core::$t::consts::TAU
            }
            #[inline]
            fn LOG10_2() -> $t {
                core::$t::consts::LOG10_2
            }
            #[inline]
            fn LOG2_10() -> $t {
                core::$t::consts::LOG2_10
            }
        }
    };
}
float_impls!(f32, 8, 23);
float_impls!(f64, 11, 52);

/// num-traits' `integer_decode`: the significand with its implicit bit (a
/// subnormal's shifted left once), the exponent of its last bit, and the sign
/// as ±1, for a float with `exponent_bits` and `mantissa_bits` fields.
fn integer_decode(bits: u64, exponent_bits: u32, mantissa_bits: u32) -> (u64, i16, i8) {
    let sign: i8 = if bits >> (exponent_bits + mantissa_bits) == 0 {
        1
    } else {
        -1
    };
    let fraction = bits & ((1 << mantissa_bits) - 1);
    let exponent = ((bits >> mantissa_bits) & ((1 << exponent_bits) - 1)) as i16;
    let mantissa = if exponent == 0 {
        fraction << 1
    } else {
        fraction | (1 << mantissa_bits)
    };
    let bias = (1i16 << (exponent_bits - 1)) - 1;
    (mantissa, exponent - (bias + mantissa_bits as i16), sign)
}
