//! libm's functions against 60-digit references (tests/reference.rs, from
//! tools/reference.py), and rint's exact rounding.

mod reference;

/// Distance in units in the last place between two finite f64 of one sign.
fn ulps(a: f64, b: f64) -> u64 {
    assert_eq!(a.is_sign_negative(), b.is_sign_negative(), "{a:e} vs {b:e}");
    a.to_bits().abs_diff(b.to_bits())
}

#[test]
fn asinh_within_one_ulp() {
    let mut worst = 0;
    for &(x, expected) in reference::ASINH {
        let (x, expected) = (f64::from_bits(x), f64::from_bits(expected));
        let error = ulps(libm::asinh(x), expected);
        assert!(
            error <= 1,
            "asinh({x:e}) = {:e}, expected {expected:e}",
            libm::asinh(x)
        );
        worst = worst.max(error);
    }
    println!(
        "asinh: {} references, worst {worst} ulp",
        reference::ASINH.len()
    );
}

#[test]
fn acosh_within_one_ulp() {
    let mut worst = 0;
    for &(x, expected) in reference::ACOSH {
        let (x, expected) = (f64::from_bits(x), f64::from_bits(expected));
        let error = ulps(libm::acosh(x), expected);
        assert!(
            error <= 1,
            "acosh({x:e}) = {:e}, expected {expected:e}",
            libm::acosh(x)
        );
        worst = worst.max(error);
    }
    println!(
        "acosh: {} references, worst {worst} ulp",
        reference::ACOSH.len()
    );
}

#[test]
fn special_values() {
    assert_eq!(libm::asinh(0.0).to_bits(), 0f64.to_bits());
    assert_eq!(libm::asinh(-0.0).to_bits(), (-0f64).to_bits());
    assert_eq!(libm::asinh(f64::INFINITY), f64::INFINITY);
    assert_eq!(libm::asinh(f64::NEG_INFINITY), f64::NEG_INFINITY);
    assert!(libm::asinh(f64::NAN).is_nan());
    assert_eq!(libm::acosh(1.0).to_bits(), 0f64.to_bits());
    assert_eq!(libm::acosh(f64::INFINITY), f64::INFINITY);
    assert!(libm::acosh(0.5).is_nan());
    assert!(libm::acosh(-1.0).is_nan());
    assert!(libm::acosh(f64::NAN).is_nan());
}

#[test]
fn rint_rounds_ties_to_even() {
    for (x, r) in [
        (0.5, 0.0),
        (1.5, 2.0),
        (2.5, 2.0),
        (-0.5, -0.0),
        (-1.5, -2.0),
        (0.49999999999999994, 0.0),
        (4503599627370495.5, 4503599627370496.0),
        (1e300, 1e300),
        (-3.7, -4.0),
        (3.2, 3.0),
    ] {
        assert_eq!(libm::rint(x).to_bits(), f64::to_bits(r), "rint({x})");
        assert_eq!(
            libm::rintf(x as f32).to_bits(),
            (r as f32).to_bits(),
            "rintf({x})"
        );
    }
    assert!(libm::rint(f64::NAN).is_nan());
    assert_eq!(libm::rintf(f32::INFINITY), f32::INFINITY);
}
