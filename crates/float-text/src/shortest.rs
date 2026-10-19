//! The shortest decimal that rounds back to a finite, positive f64, with one
//! 128 × 64-bit product (the method zmij uses, after Xiang JunBo's
//! refinement of Schubfach).
//!
//! For `v = c × 2^e`, let `k = floor(log10 2^e)` and scale by 10^-(k+1):
//! `v × 10^-(k+1) = integral + fraction` has 15 or 16 digits before the
//! point (the "shorter" precision) and a 64-bit fraction. Half an ulp of `v`
//! in the same units is under 1/2 and at least 1/20, so:
//!
//! - when `integral` or `integral + 1` lies inside the rounding interval
//!   (the fraction is within half an ulp of either end), that is the result
//!   at 10^(k+1); it is the only one there, and a still shorter decimal would
//!   be one of these two with trailing zeros;
//! - otherwise one more digit, `round(fraction × 10)`, gives the closest
//!   decimal at 10^k, which the interval (at least a tenth wide) holds.
//!
//! The power of ten is a 128-bit significand rounded down; the product is
//! truncated, and the small biases below cover both errors. A power of two
//! (its interval is narrower below) takes the same steps with the lower
//! half-width halved.

use crate::tables::{POW10, POW10_MIN};

/// Extra fraction bits kept below the integral part while multiplying.
const EXTRA: u32 = 6;

/// floor(log10 2^e) for e in -1334..=2620.
#[inline(always)]
const fn floor_log10_pow2(e: i32) -> i32 {
    (e * 315_653) >> 20
}

/// floor(log10 (3/4 × 2^e)) for e in -1334..=2620.
#[inline(always)]
const fn floor_log10_three_quarters_pow2(e: i32) -> i32 {
    (e * 315_653 - 131_072) >> 20
}

/// floor(log2 10^e) for e in -293..=323 (checked against the table's
/// exponents in tests/tables.rs).
#[inline(always)]
pub(crate) const fn floor_log2_pow10(e: i32) -> i32 {
    (e * 217_707) >> 16
}

/// The high 128 bits of the 192-bit product of the 128-bit `(high, low)` and `x`.
#[inline(always)]
fn product((high, low): (u64, u64), x: u64) -> (u64, u64) {
    let upper = high as u128 * x as u128;
    let carry = ((low as u128 * x as u128) >> 64) as u64;
    let lo = (upper as u64).wrapping_add(carry);
    ((upper >> 64) as u64 + (lo < upper as u64) as u64, lo)
}

/// `(x × y + c) >> 64`.
#[inline(always)]
fn mul_add_high(x: u64, y: u64, c: u64) -> u64 {
    ((x as u128 * y as u128 + c as u128) >> 64) as u64
}

/// The integral and fractional parts of `c × 2^e × 10^-(k+1)`, and half an
/// ulp of `c × 2^e` in the fraction's units.
#[inline(always)]
fn scaled(c: u64, e: i32, k: i32) -> (u64, u64, u64) {
    let exponent = -(k + 1);
    let shift = (e + floor_log2_pow10(exponent) + 1 + EXTRA as i32) as u32;
    let pow10 = POW10[(exponent - POW10_MIN) as usize];
    let (high, low) = product(pow10, c << shift);
    let integral = high >> EXTRA;
    let fraction = (high << (64 - EXTRA)) | (low >> EXTRA);
    (integral, fraction, pow10.0 >> (EXTRA + 1 - shift))
}

/// The decimal significand (perhaps with trailing zeros) and power of ten of
/// the shortest decimal for the f64 with these biased exponent and mantissa
/// fields (not zero, not infinite or NaN).
#[inline]
pub(crate) fn shortest(exponent_bits: u32, mantissa_bits: u64) -> (u64, u8, i32) {
    let (c, e) = if exponent_bits == 0 {
        (mantissa_bits, 1 - 1075)
    } else {
        (mantissa_bits | 1 << 52, exponent_bits as i32 - 1075)
    };
    if exponent_bits != 0 && mantissa_bits == 0 {
        return power_of_two(c, e);
    }
    let k = floor_log10_pow2(e);
    let (integral, fraction, half_ulp) = scaled(c, e, k);
    // An even significand's interval includes its ends.
    let half_ulp = half_ulp + (1 - (c & 1));
    let round_up = fraction.wrapping_add(half_ulp) < fraction;
    let round_down = half_ulp > fraction;
    if round_up | round_down {
        return (integral + round_up as u64, 0, k + 1);
    }
    // The extra digit, rounded to nearest (ties to even: x.25 at the shorter
    // precision is the only exact tie that rounds down).
    let mut digit = mul_add_high(fraction, 10, (1 << 63) + 6);
    if fraction == 1 << 62 {
        digit = 2;
    }
    (integral, digit as u8 + b'0', k + 1)
}

/// [`shortest`] for a power of two: the interval below is half as wide.
/// (Common in models: 0.5, 1, 2, -1 in normals and grids.)
#[inline]
fn power_of_two(c: u64, e: i32) -> (u64, u8, i32) {
    let k = floor_log10_three_quarters_pow2(e);
    let (integral, fraction, half_ulp) = scaled(c, e, k);
    let round_up = half_ulp > u64::MAX - fraction;
    let round_down = (half_ulp >> 1) > fraction;
    if round_up | round_down {
        return (integral + round_up as u64, 0, k + 1);
    }
    let nearest = mul_add_high(fraction, 10, (1 << 63) - 1);
    let lowest = mul_add_high(fraction.wrapping_sub(half_ulp >> 1), 10, !0);
    (integral, nearest.max(lowest) as u8 + b'0', k + 1)
}
