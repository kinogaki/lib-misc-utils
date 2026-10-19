//! The conversion rules naga's constant evaluator relies on.

use num_traits::{Float, FloatCore, FromPrimitive, NumCast, ToPrimitive, cast};

#[test]
fn float_to_int_truncates_and_checks_range() {
    assert_eq!(2.9f64.to_i32(), Some(2));
    assert_eq!((-2.9f64).to_i32(), Some(-2));
    assert_eq!((-0.9f64).to_u32(), Some(0));
    assert_eq!((-1.0f64).to_u32(), None);
    assert_eq!(f64::NAN.to_i32(), None);
    assert_eq!(f64::INFINITY.to_u64(), None);
    assert_eq!(f64::NEG_INFINITY.to_i64(), None);
    // Signed: [MIN, MAX] after truncation.
    assert_eq!(2147483647.9f64.to_i32(), Some(i32::MAX));
    assert_eq!(2147483648.0f64.to_i32(), None);
    assert_eq!((-2147483648.9f64).to_i32(), Some(i32::MIN));
    assert_eq!((-2147483649.0f64).to_i32(), None);
    assert_eq!(127.5f32.to_i8(), Some(127));
    assert_eq!(128.0f32.to_i8(), None);
    assert_eq!((-128.99f32).to_i8(), Some(-128));
    assert_eq!(255.99f64.to_u8(), Some(255));
    assert_eq!(256.0f64.to_u8(), None);
    // Where the float cannot hold MAX exactly, the next float up is MAX + 1.
    assert_eq!(2147483520.0f32.to_i32(), Some(2147483520));
    assert_eq!(2147483648.0f32.to_i32(), None);
    assert_eq!(9223372036854774784.0f64.to_i64(), Some(9223372036854774784));
    assert_eq!(9223372036854775808.0f64.to_i64(), None);
    assert_eq!((-9223372036854775808.0f64).to_i64(), Some(i64::MIN));
    assert_eq!(
        18446744073709549568.0f64.to_u64(),
        Some(18446744073709549568)
    );
    assert_eq!(18446744073709551616.0f64.to_u64(), None);
    assert_eq!(4294967040.0f32.to_u32(), Some(4294967040));
    assert_eq!(4294967296.0f32.to_u32(), None);
}

#[test]
fn int_conversions_check_range() {
    assert_eq!(300i32.to_u8(), None);
    assert_eq!((-1i64).to_u64(), None);
    assert_eq!(u64::MAX.to_i64(), None);
    assert_eq!(u64::MAX.to_u128(), Some(u64::MAX as u128));
    assert_eq!(i64::MIN.to_i32(), None);
    assert_eq!(i32::from_i64(-5), Some(-5));
    assert_eq!(u16::from_u64(70000), None);
    assert_eq!(u32::from_f64(3.99), Some(3));
    assert_eq!(i16::from_f32(-32768.5), Some(-32768));
    assert_eq!(i16::from_f32(32768.0), None);
}

#[test]
fn to_float_rounds_like_as() {
    assert_eq!(i64::MAX.to_f32(), Some(i64::MAX as f32));
    assert_eq!(16777217i32.to_f32(), Some(16777216.0));
    assert_eq!(u64::MAX.to_f64(), Some(u64::MAX as f64));
    assert_eq!(1e300f64.to_f32(), Some(f32::INFINITY));
    assert_eq!(0.1f64.to_f32(), Some(0.1f32));
    assert_eq!(0.1f32.to_f64(), Some(0.1f32 as f64));
    assert!(f64::NAN.to_f32().unwrap().is_nan());
    assert_eq!(f32::from_i64(i64::MIN), Some(i64::MIN as f32));
    assert_eq!(f64::from_u64(u64::MAX), Some(u64::MAX as f64));
}

#[test]
fn num_cast_keeps_fractions_between_floats() {
    assert_eq!(<f32 as NumCast>::from(2.5f64), Some(2.5f32));
    assert_eq!(<f64 as NumCast>::from(-0.25f32), Some(-0.25));
    assert_eq!(<i32 as NumCast>::from(2.5f64), Some(2));
    assert_eq!(cast::<u8, f64>(200), Some(200.0));
    assert_eq!(cast::<f64, u8>(256.0), None);
}

#[test]
fn float_traits_are_std() {
    let x = 0.7f64;
    assert_eq!(Float::sin(x).to_bits(), x.sin().to_bits());
    assert_eq!(Float::powf(x, 2.5).to_bits(), x.powf(2.5).to_bits());
    assert_eq!(Float::abs_sub(1.0f64, 3.0), 0.0);
    assert_eq!(Float::abs_sub(3.0f64, 1.0), 2.0);
    assert_eq!(FloatCore::integer_decode(1.0f64), (1 << 52, -52, 1));
    assert_eq!(FloatCore::integer_decode(-0.5f32), (1 << 23, -24, -1));
    assert_eq!(FloatCore::integer_decode(f64::from_bits(1)), (2, -1075, 1));
    assert_eq!(FloatCore::integer_decode(0.0f32), (0, -150, 1));
}
