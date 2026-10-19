//! The eight-digit SWAR conversion for every value it accepts.

#[test]
fn eight_digits_for_all_of_0_to_10_pow_8() {
    let step = if cfg!(debug_assertions) { 997 } else { 1 };
    for v in (0..100_000_000u32)
        .step_by(step)
        .chain([99_999_999, 10_000_000, 9_999_999, 0])
    {
        let bytes = kinogaki_float_text::eight_digits(v).to_le_bytes();
        assert_eq!(&bytes, format!("{v:08}").as_bytes(), "{v}");
    }
}
