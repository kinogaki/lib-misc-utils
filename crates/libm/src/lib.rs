//! Kinogaki's own `libm`, published under the crates.io name so the frozen
//! naga resolves to it through `[patch.crates-io]`. It is not upstream libm:
//! it has only the four functions naga calls directly (its other float maths
//! goes through our `num-traits`, which uses std).
//!
//! - [`rint`], [`rintf`]: round to the nearest integer, ties to even (exact).
//! - [`asinh`], [`acosh`]: the classic argument-range split (as in fdlibm and
//!   musl) over std's `ln`, `ln_1p` and `sqrt`, so large arguments do not
//!   overflow and small ones keep their precision. Accuracy is checked against
//!   reference values computed to 40 digits (tests/accuracy.rs): within 1 ulp.

use std::f64::consts::LN_2;

/// Round to the nearest integer, ties to even.
#[inline]
pub fn rint(x: f64) -> f64 {
    x.round_ties_even()
}

/// Round to the nearest integer, ties to even.
#[inline]
pub fn rintf(x: f32) -> f32 {
    x.round_ties_even()
}

/// The inverse hyperbolic sine.
pub fn asinh(x: f64) -> f64 {
    let a = x.abs();
    let y = if a >= 2f64.powi(26) || a.is_nan() {
        // sqrt(a² + 1) ≈ a, so asinh(a) = ln(2a) = ln(a) + ln 2 (also NaN and ∞).
        a.ln() + LN_2
    } else if a >= 2.0 {
        // ln(a + sqrt(a² + 1)) written as ln(2a + 1 / (sqrt(a² + 1) + a)).
        (2.0 * a + 1.0 / ((a * a + 1.0).sqrt() + a)).ln()
    } else if a >= 2f64.powi(-26) {
        // ln(1 + t) with t = a + a² / (sqrt(a² + 1) + 1), accurate near 0.
        let a2 = a * a;
        (a + a2 / ((a2 + 1.0).sqrt() + 1.0)).ln_1p()
    } else {
        // asinh(a) = a - a³/6 + ...: a is the correctly rounded result.
        a
    };
    y.copysign(x)
}

/// The inverse hyperbolic cosine (NaN below 1).
pub fn acosh(x: f64) -> f64 {
    if x.is_nan() || x < 1.0 {
        f64::NAN
    } else if x < 2.0 {
        // ln(1 + t) with t = (x - 1) + sqrt((x - 1)² + 2(x - 1)), accurate near 1.
        let t = x - 1.0;
        (t + (t * t + 2.0 * t).sqrt()).ln_1p()
    } else if x < 2f64.powi(26) {
        // ln(x + sqrt(x² - 1)) written as ln(2x - 1 / (x + sqrt(x² - 1))).
        (2.0 * x - 1.0 / (x + (x * x - 1.0).sqrt())).ln()
    } else {
        // sqrt(x² - 1) ≈ x: ln(2x) = ln(x) + ln 2 (also ∞).
        x.ln() + LN_2
    }
}
