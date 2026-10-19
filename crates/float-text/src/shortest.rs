//! The shortest decimal that rounds back to a finite, positive f64: Raffaello
//! Giulietti's Schubfach ("The Schubfach way to render doubles", 2020).
//!
//! For `v = c × 2^q`, the rounding interval is `[cbl, cbr] / 4 × 2^q` around
//! `cb / 4 × 2^q` (`cb = 4c`). One 126-bit power of ten `g` scales the three
//! to a precision `10^k` at which the interval is narrower than 10 units:
//! `vbl < vb < vbr` (times 4, rounded to odd, so comparisons stay exact).
//! Then the interval holds at most one multiple of 10 (a decimal one digit
//! shorter), and otherwise the result is `s = floor(vb / 4)` or `s + 1`,
//! whichever is in the interval, or the closer (ties to even) when both are.
//! Bounds count only for an even `c` (round-half-even reads them back).

use crate::tables::{G, G_MIN};

const PRECISION: u32 = 53;
/// The exponent of the smallest subnormal's last bit.
const Q_MIN: i32 = -1074;
/// The smallest normal significand, 2^52.
const C_MIN: u64 = 1 << 52;

/// floor(q × log10(2)), exact for |q| <= 5456721.
#[inline]
fn flog10_pow2(q: i32) -> i32 {
    ((q as i64 * 661_971_961_083) >> 41) as i32
}

/// floor(q × log10(3/4 × 2)), exact for |q| <= 5456721.
#[inline]
fn flog10_three_quarters_pow2(q: i32) -> i32 {
    ((q as i64 * 661_971_961_083 - 274_743_187_321) >> 41) as i32
}

/// floor(e × log2(10)), exact for |e| <= 1233.
#[inline]
fn flog2_pow10(e: i32) -> i32 {
    ((e as i64 * 913_124_641_741) >> 38) as i32
}

#[inline]
fn mul_high(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) >> 64) as u64
}

/// `g × cp / 2^127` rounded to odd (the low bit records an inexact result),
/// for the 126-bit `g = g1 × 2^63 + g0`.
#[inline]
fn round_to_odd(g1: u64, g0: u64, cp: u64) -> u64 {
    let x1 = mul_high(g0, cp);
    let y0 = g1.wrapping_mul(cp);
    let y1 = mul_high(g1, cp);
    let z = (y0 >> 1).wrapping_add(x1);
    let vbp = y1 + (z >> 63);
    vbp | (((z & (u64::MAX >> 1)) + (u64::MAX >> 1)) >> 63)
}

/// The decimal significand (perhaps with trailing zeros) and power of ten of
/// the shortest decimal for the f64 with these biased exponent and mantissa
/// fields (not zero, not infinite or NaN).
#[inline]
pub(crate) fn shortest(exponent_bits: u32, mantissa_bits: u64) -> (u64, i32) {
    if exponent_bits != 0 {
        let mq = -Q_MIN + 1 - exponent_bits as i32;
        let c = C_MIN | mantissa_bits;
        // An integer below 2^53 is its own shortest decimal.
        if 0 < mq && mq < PRECISION as i32 {
            let f = c >> mq;
            if f << mq == c {
                return (f, 0);
            }
        }
        return to_decimal(-mq, c, 0);
    }
    match mantissa_bits {
        // The two smallest subnormals' intervals are too wide for one scaled
        // pass to be sure of the shortest decimal; theirs are known.
        1 => (5, -324),
        2 => (1, -323),
        _ => to_decimal(Q_MIN, mantissa_bits, 0),
    }
}

#[inline]
fn to_decimal(q: i32, c: u64, dk: i32) -> (u64, i32) {
    let out = c & 1;
    let cb = c << 2;
    let cbr = cb + 2;
    // At a power of two the interval below is half as wide.
    let (cbl, k) = if c != C_MIN || q == Q_MIN {
        (cb - 2, flog10_pow2(q))
    } else {
        (cb - 1, flog10_three_quarters_pow2(q))
    };
    let h = q + flog2_pow10(-k) + 2;
    let (g1, g0) = G[(-k - G_MIN) as usize];
    let vb = round_to_odd(g1, g0, cb << h);
    let vbl = round_to_odd(g1, g0, cbl << h);
    let vbr = round_to_odd(g1, g0, cbr << h);
    let s = vb >> 2;
    if s >= 100 {
        // A decimal one digit shorter: the multiples of 10 around s.
        let sp10 = 10 * (s / 10);
        let tp10 = sp10 + 10;
        let upin = vbl + out <= sp10 << 2;
        let wpin = (tp10 << 2) + out <= vbr;
        if upin != wpin {
            return (if upin { sp10 } else { tp10 }, k);
        }
    }
    let t = s + 1;
    let uin = vbl + out <= s << 2;
    let win = (t << 2) + out <= vbr;
    if uin != win {
        return (if uin { s } else { t }, k + dk);
    }
    // Both in the interval: the closer, ties to even.
    let mid = (s + t) << 1;
    let pick = if vb < mid || (vb == mid && s & 1 == 0) {
        s
    } else {
        t
    };
    (pick, k + dk)
}
