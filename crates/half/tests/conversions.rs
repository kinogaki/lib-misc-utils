//! f16 conversions against IEEE 754, over all 65,536 bit patterns and every
//! rounding boundary between neighbouring f16 values.
//!
//! The reference does not use the code under test: an f16's value is
//! `(-1)^s × m × 2^(e - 25)` computed exactly in f64, and rounding is decided
//! by comparing against the exact midpoints between neighbouring f16 values.

use half::f16;

/// The exact value of a finite f16, from its fields.
fn exact(bits: u16) -> f64 {
    let sign = if bits & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exponent = ((bits >> 10) & 0x1f) as i32;
    let mantissa = (bits & 0x3ff) as f64;
    let magnitude = if exponent == 0 {
        mantissa * 2f64.powi(-24)
    } else {
        (1024.0 + mantissa) * 2f64.powi(exponent - 25)
    };
    sign * magnitude
}

fn is_nan_bits(bits: u16) -> bool {
    bits & 0x7c00 == 0x7c00 && bits & 0x3ff != 0
}

#[test]
fn every_f16_widens_exactly() {
    for bits in 0..=u16::MAX {
        let value = f16::from_bits(bits);
        let wide32 = value.to_f32();
        let wide64 = value.to_f64();
        if is_nan_bits(bits) {
            assert!(wide32.is_nan() && wide64.is_nan(), "{bits:#06x}");
            // Payload's top bits kept, quiet bit set, sign kept.
            assert_eq!(
                wide32.to_bits() >> 13 & 0x3ff,
                (bits & 0x3ff) as u32 | 0x200,
                "{bits:#06x}"
            );
            assert_eq!(wide32.is_sign_negative(), bits & 0x8000 != 0);
            assert_eq!(
                wide64.to_bits() >> 42 & 0x3ff,
                (bits & 0x3ff) as u64 | 0x200,
                "{bits:#06x}"
            );
            continue;
        }
        if bits & 0x7fff == 0x7c00 {
            let infinity = if bits & 0x8000 != 0 {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
            assert_eq!(wide64, infinity);
            assert_eq!(wide32 as f64, infinity);
            continue;
        }
        let expected = exact(bits);
        assert_eq!(wide64.to_bits(), expected.to_bits(), "{bits:#06x}");
        assert_eq!((wide32 as f64).to_bits(), expected.to_bits(), "{bits:#06x}");
    }
}

#[test]
fn every_f16_round_trips() {
    for bits in 0..=u16::MAX {
        let value = f16::from_bits(bits);
        let back32 = f16::from_f32(value.to_f32()).to_bits();
        let back64 = f16::from_f64(value.to_f64()).to_bits();
        if is_nan_bits(bits) {
            // A signalling NaN comes back quiet; nothing else changes.
            assert_eq!(back32, bits | 0x200, "{bits:#06x}");
            assert_eq!(back64, bits | 0x200, "{bits:#06x}");
        } else {
            assert_eq!(back32, bits, "{bits:#06x}");
            assert_eq!(back64, bits, "{bits:#06x}");
        }
    }
}

/// Every pair of neighbouring non-negative finite f16 values, and the step
/// from f16::MAX to infinity: the exact midpoint goes to the even one, and the
/// nearest f32/f64 values on either side of it go to their side.
#[test]
fn every_rounding_boundary() {
    for low in 0..0x7c00u16 {
        let high = low + 1;
        let a = exact(low);
        // f16::MAX's upper neighbour is where 2^16 would be; at or above its
        // midpoint (65520) a value overflows to infinity.
        let b = if high == 0x7c00 { 65536.0 } else { exact(high) };
        let mid = (a + b) / 2.0; // exact: a and b are close binary fractions
        let even = if low % 2 == 0 { low } else { high };
        for sign in [0u16, 0x8000] {
            let s = if sign == 0 { 1.0 } else { -1.0 };
            let check64 = |value: f64, expected: u16| {
                assert_eq!(
                    f16::from_f64(value).to_bits(),
                    expected | sign,
                    "f64 {value:e} between {low:#06x} and {high:#06x}"
                );
            };
            check64(s * mid, even);
            check64(s * f64::from_bits(mid.to_bits() - 1), low);
            check64(s * f64::from_bits(mid.to_bits() + 1), high);
            check64(s * a, low);
            let mid32 = mid as f32;
            assert_eq!(mid32 as f64, mid, "midpoints are exact in f32");
            let check32 = |value: f32, expected: u16| {
                assert_eq!(
                    f16::from_f32(value).to_bits(),
                    expected | sign,
                    "f32 {value:e} between {low:#06x} and {high:#06x}"
                );
            };
            check32(s as f32 * mid32, even);
            check32(s as f32 * f32::from_bits(mid32.to_bits() - 1), low);
            check32(s as f32 * f32::from_bits(mid32.to_bits() + 1), high);
        }
    }
}

#[test]
fn below_the_smallest_subnormal() {
    let tiny = 2f64.powi(-24); // f16::MIN_POSITIVE_SUBNORMAL
    assert_eq!(f16::from_f64(tiny / 2.0).to_bits(), 0); // tie to even (zero)
    assert_eq!(
        f16::from_f64(f64::from_bits((tiny / 2.0).to_bits() + 1)).to_bits(),
        1
    );
    assert_eq!(f16::from_f64(tiny / 4.0).to_bits(), 0);
    assert_eq!(f16::from_f64(-tiny / 4.0).to_bits(), 0x8000);
    assert_eq!(f16::from_f64(f64::MIN_POSITIVE).to_bits(), 0);
    assert_eq!(f16::from_f64(5e-324).to_bits(), 0);
    assert_eq!(f16::from_f32(f32::from_bits(1)).to_bits(), 0);
    assert_eq!(f16::from_f32(f32::MIN_POSITIVE).to_bits(), 0);
    assert_eq!(f16::from_f32(-0.0).to_bits(), 0x8000);
}

#[test]
fn overflow_and_special_values() {
    assert_eq!(f16::from_f32(65519.996).to_bits(), 0x7bff);
    assert_eq!(f16::from_f32(65520.0).to_bits(), 0x7c00);
    assert_eq!(f16::from_f64(1e300).to_bits(), 0x7c00);
    assert_eq!(f16::from_f64(-1e300).to_bits(), 0xfc00);
    assert_eq!(f16::from_f32(f32::INFINITY).to_bits(), 0x7c00);
    assert_eq!(f16::from_f32(f32::NEG_INFINITY).to_bits(), 0xfc00);
    assert!(f16::from_f32(f32::NAN).is_nan());
    assert!(f16::from_f64(f64::NAN).is_nan());
    // A NaN whose payload sits below the bits f16 keeps stays a NaN.
    assert!(f16::from_f32(f32::from_bits(0x7f80_0001)).is_nan());
    assert!(f16::from_f64(f64::from_bits(0x7ff0_0000_0000_0001)).is_nan());
}

/// Random f32 and f64 inputs against the nearest-value reference.
#[test]
fn random_values_round_to_nearest_even() {
    let finite: Vec<f64> = (0..0x7c00u16).map(exact).collect();
    let reference = |value: f64| -> u16 {
        let sign = if value.is_sign_negative() { 0x8000 } else { 0 };
        let a = value.abs();
        if a >= 65520.0 {
            return sign | 0x7c00;
        }
        let i = finite.partition_point(|&v| v <= a) - 1; // finite[i] <= a
        if finite[i] == a || i + 1 == finite.len() {
            return sign | i as u16;
        }
        let (low, high) = (finite[i], finite[i + 1]);
        let mid = (low + high) / 2.0;
        let pick = if a < mid {
            i
        } else if a > mid {
            i + 1
        } else if i % 2 == 0 {
            i
        } else {
            i + 1
        };
        sign | pick as u16
    };
    let mut state = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..2_000_000 {
        let r = next();
        // Bias towards f16's range: exponents near 2^-26..2^17.
        let x32 = f32::from_bits((r as u32 & 0x807f_ffff) | ((100 + (r >> 40) % 48) as u32) << 23);
        assert_eq!(
            f16::from_f32(x32).to_bits(),
            reference(x32 as f64),
            "{x32:e}"
        );
        let x64 = f64::from_bits((r & 0x800f_ffff_ffff_ffff) | ((996 + (r >> 52) % 48) << 52));
        assert_eq!(f16::from_f64(x64).to_bits(), reference(x64), "{x64:e}");
    }
}

#[test]
fn arithmetic_is_correctly_rounded() {
    // a op b computed exactly in f64 (all are exact there for f16 operands
    // except division), then rounded once with the f64 path.
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..1_000_000 {
        let r = next();
        let a = f16::from_bits(r as u16);
        let b = f16::from_bits((r >> 16) as u16);
        if a.is_nan() || b.is_nan() {
            assert!((a + b).is_nan() && (a * b).is_nan());
            continue;
        }
        let (x, y) = (a.to_f64(), b.to_f64());
        assert_eq!(
            (a + b).to_bits(),
            f16::from_f64(x + y).to_bits(),
            "{a} + {b}"
        );
        assert_eq!(
            (a - b).to_bits(),
            f16::from_f64(x - y).to_bits(),
            "{a} - {b}"
        );
        assert_eq!(
            (a * b).to_bits(),
            f16::from_f64(x * y).to_bits(),
            "{a} * {b}"
        );
        let q = a / b;
        let exact_q = f16::from_f64(x / y);
        if q.is_nan() {
            assert!(exact_q.is_nan());
        } else {
            assert_eq!(q.to_bits(), exact_q.to_bits(), "{a} / {b}");
        }
        let rem = a % b;
        if rem.is_nan() {
            assert!((x % y).is_nan());
        } else {
            assert_eq!(rem.to_bits(), f16::from_f64(x % y).to_bits(), "{a} % {b}");
        }
    }
}

#[test]
fn comparisons_and_text() {
    assert_eq!(f16::from_f32(0.0), f16::from_f32(-0.0));
    assert!(f16::NAN != f16::NAN);
    assert!(f16::NAN.partial_cmp(&f16::ONE).is_none());
    assert!(f16::NEG_ONE < f16::ZERO && f16::ZERO < f16::MIN_POSITIVE_SUBNORMAL);
    assert!(f16::MIN < f16::NEG_ONE && f16::MAX < f16::INFINITY);
    assert_eq!(f16::from_f32(0.1).to_string(), "0.099975586");
    assert_eq!(f16::ONE.to_string(), "1");
    assert_eq!(format!("{:?}", f16::from_f32(1.5)), "1.5");
    assert_eq!(f16::MAX.to_f32(), 65504.0);
    assert_eq!(f16::EPSILON.to_f32(), 2f32.powi(-10));
    assert_eq!(f16::MIN_POSITIVE.to_f32(), 2f32.powi(-14));
    assert_eq!(f16::from_f32_const(-1.0).to_bits(), 0xbc00);
    assert_eq!(
        [f16::ONE, f16::ONE, f16::from_f32(2.0)]
            .into_iter()
            .sum::<f16>()
            .to_f32(),
        4.0
    );
    assert_eq!(
        [f16::from_f32(3.0), f16::from_f32(2.0)]
            .into_iter()
            .product::<f16>()
            .to_f32(),
        6.0
    );
}
