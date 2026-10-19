//! The digits are std's shortest round-trip digits (`{:e}`) except at exact
//! ties, where std rounds up and we round to even (as Ryu and zmij do); the
//! layout is the documented one, and the text parses back to the same bits.

// Some inputs are deliberately spelled past f64 precision (they round to the boundary values).
#![allow(clippy::excessive_precision)]

use kinogaki_float_text::FloatText;

/// std's shortest digits and scientific exponent of a finite, nonzero value.
fn std_digits(value: f64) -> (String, i32) {
    let text = format!("{:e}", value.abs());
    let (mantissa, exponent) = text.split_once('e').unwrap();
    (mantissa.replace('.', ""), exponent.parse().unwrap())
}

/// The documented layout of `digits × 10^(exponent - len + 1)`.
fn layout(negative: bool, digits: &str, exponent: i32) -> String {
    let sign = if negative { "-" } else { "" };
    let n = digits.len() as i32;
    if (0..=15).contains(&exponent) {
        let whole = exponent + 1;
        if n <= whole {
            format!("{sign}{digits}{}.0", "0".repeat((whole - n) as usize))
        } else {
            format!(
                "{sign}{}.{}",
                &digits[..whole as usize],
                &digits[whole as usize..]
            )
        }
    } else if (-5..0).contains(&exponent) {
        format!("{sign}0.{}{digits}", "0".repeat((-exponent - 1) as usize))
    } else {
        let rest = if n > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        let e_sign = if exponent < 0 { '-' } else { '+' };
        format!(
            "{sign}{}{rest}e{e_sign}{}",
            &digits[..1],
            exponent.unsigned_abs()
        )
    }
}

fn check(buffer: &mut FloatText, value: f64) {
    let text = buffer.format(value).to_string();
    let back: f64 = text.parse().unwrap();
    assert_eq!(back.to_bits(), value.to_bits(), "{value:e} -> {text}");
    if value != 0.0 {
        let (mut digits, exponent) = std_digits(value);
        let expected = layout(value < 0.0, &digits, exponent);
        if text != expected {
            // Only an exact tie may differ: the exact value is one digit
            // longer and ends in 5; we keep the even digit, std rounds up.
            let exact = format!("{:.60e}", value.abs());
            let exact_digits = exact.split_once('e').unwrap().0.replace('.', "");
            let last = digits.pop().unwrap().to_digit(10).unwrap();
            let even = last - 1;
            assert!(even % 2 == 0, "{value:e}: {text} vs std {expected}");
            digits.push(char::from_digit(even, 10).unwrap());
            let tie = format!("{digits}5");
            assert!(
                exact_digits.starts_with(&tie)
                    && exact_digits[tie.len()..].bytes().all(|b| b == b'0'),
                "{value:e}: {text} vs std {expected}"
            );
            assert_eq!(text, layout(value < 0.0, &digits, exponent), "{value:e}");
        }
    }
}

#[test]
fn special_and_boundary_values() {
    let mut buffer = FloatText::new();
    for value in [
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.1,
        0.2,
        0.3,
        1.0 / 3.0,
        2.0 / 3.0,
        1e-5,
        1e-6,
        1.5e-5,
        9.999999999999999e15,
        1e15,
        1e16,
        1e21,
        1e22,
        1e23,
        1e300,
        1e-300,
        5e-324,
        1e-323,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::EPSILON,
        9007199254740992.0,
        9007199254740993.0,
        123456789.0,
        0.000123,
        299792458.0,
        6.02214076e23,
        1.602176634e-19,
        4.35,
        2.675,
    ] {
        check(&mut buffer, value);
        check(&mut buffer, -value);
    }
    assert_eq!(buffer.format(f64::NAN), "NaN");
    assert_eq!(buffer.format(f64::INFINITY), "inf");
    assert_eq!(buffer.format(f64::NEG_INFINITY), "-inf");
    for (value, text) in [
        (1e16, "1e+16"),
        (1.5e-5, "0.000015"),
        (1e-6, "1e-6"),
        (100.0, "100.0"),
        (123.456, "123.456"),
        (f64::MAX, "1.7976931348623157e+308"),
        (5e-324, "5e-324"),
        (9007199254740992.0, "9007199254740992.0"),
    ] {
        assert_eq!(buffer.format(value), text);
    }
}

/// Every power of two and its neighbours (the interval is asymmetric there),
/// and every subnormal exponent.
#[test]
fn powers_of_two_and_neighbours() {
    let mut buffer = FloatText::new();
    for exponent in 0..2047u64 {
        for mantissa in [0, 1, 2, (1 << 52) - 1, (1 << 51)] {
            let value = f64::from_bits(exponent << 52 | mantissa);
            if value.is_finite() {
                check(&mut buffer, value);
            }
        }
    }
}

#[test]
fn random_bit_patterns() {
    let mut buffer = FloatText::new();
    let mut state = 0x853c_49e6_748f_ea9bu64;
    let count = if cfg!(debug_assertions) {
        300_000
    } else {
        5_000_000
    };
    for _ in 0..count {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let value = f64::from_bits(state);
        if value.is_finite() {
            check(&mut buffer, value);
        }
    }
}

/// Decimals with few digits, the kind exporters write (`%.6f`), all exponents.
#[test]
fn short_decimals() {
    let mut buffer = FloatText::new();
    for exponent in -320..=308 {
        for mantissa in [1u64, 5, 12, 125, 999, 123456, 1000001, 31415926] {
            let value: f64 = format!("{mantissa}e{exponent}").parse().unwrap();
            if value.is_finite() && value != 0.0 {
                check(&mut buffer, value);
            }
        }
    }
}
