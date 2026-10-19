//! The sixteen-digit conversion against `format!` for every lane value.

#[test]
fn every_lane_value() {
    let step = if cfg!(debug_assertions) { 997 } else { 1 };
    for v in (0..100_000_000u32)
        .step_by(step)
        .chain([99_999_999, 10_000_000, 9_999_999, 0])
    {
        let w = 99_999_999 - v;
        let (a, b) = kinogaki_float_text::sixteen_digits(v, w);
        let mut text = [0u8; 16];
        text[..8].copy_from_slice(&a.to_le_bytes());
        text[8..].copy_from_slice(&b.to_le_bytes());
        let mut expected = [0u8; 16];
        let (mut x, mut y) = (v, w);
        for i in (0..8).rev() {
            expected[i] = b'0' + (x % 10) as u8;
            expected[8 + i] = b'0' + (y % 10) as u8;
            x /= 10;
            y /= 10;
        }
        assert_eq!(text, expected, "{v}");
    }
}
