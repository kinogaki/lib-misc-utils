//! Sixteen ASCII digits at a time: two numbers below 10^8 to two
//! little-endian words of eight digits, in `u64` arithmetic. (A NEON
//! version measured no faster where the text is copied into a buffer, as
//! lib-obj does, so there is none.)

/// The digits of `high` and `low` (each below 10^8), eight per word.
#[inline]
pub fn sixteen_digits(high: u32, low: u32) -> (u64, u64) {
    (eight_digits(high), eight_digits(low))
}

/// The eight ASCII digits of `value` (< 10^8) as little-endian bytes, by
/// dividing several digit groups in one multiply ("SWAR"): 10^4 splits it
/// into two 32-bit lanes, 100 each of those into two 16-bit lanes, 10 each
/// of those into two bytes. Each division multiplies by a rounded-up
/// reciprocal, exact for its range, and adding `(lane - divisor) × quotient`
/// leaves the quotient above the remainder in one step.
#[inline]
fn eight_digits(value: u32) -> u64 {
    let v = value as u64;
    let x = v + ((1u64 << 32) - 10_000) * ((v * 109_951_163) >> 40);
    let x = x + ((1u64 << 16) - 100) * (((x * 5243) >> 19) & 0x0000_007f_0000_007f);
    let x = x + ((1u64 << 8) - 10) * (((x * 103) >> 10) & 0x000f_000f_000f_000f);
    // The digits are now last-first in memory order.
    x.swap_bytes() | 0x3030_3030_3030_3030
}
