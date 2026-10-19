//! The shortest decimal text that reads back as the same `f64`, fast.
//!
//! ```
//! let mut text = kinogaki_float_text::FloatText::new();
//! assert_eq!(text.format(0.1), "0.1");
//! assert_eq!(text.format(1e21), "1e+21");
//! assert_eq!(text.format(-2.0), "-2.0");
//! ```
//!
//! The digits are the shortest that round-trip, and of those the closest to
//! the exact value (ties to even): the same digits as Rust's `{:e}`
//! (tests/against_std.rs) except at exact ties, where we round to even (as
//! Ryu and zmij do). They come from Schubfach's search (src/shortest.rs).
//! The text is plain decimal when the value's scientific exponent `E` is in
//! -5..=15 (`123.456`, `0.00001`, always with a `.`: `1.0`), and `d.ddde±E`
//! outside (`1.5e+16`, `1e-6`).

mod shortest;
mod tables;

/// A reusable buffer that formats one `f64` at a time.
pub struct FloatText {
    bytes: [u8; 64],
}

impl Default for FloatText {
    fn default() -> FloatText {
        FloatText::new()
    }
}

const POW10: [u64; 18] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
];

/// Where the decimal point (or the first digit's end) goes in `bytes`: room
/// for a sign and 16 digits before it, 16 digits and an exponent after.
const POINT: usize = 24;

impl FloatText {
    pub const fn new() -> FloatText {
        FloatText { bytes: [0; 64] }
    }

    /// `value` as text (`NaN`, `inf` and `-inf` for the non-finite values).
    pub fn format(&mut self, value: f64) -> &str {
        core::str::from_utf8(self.format_bytes(value)).expect("ASCII")
    }

    /// [`FloatText::format`] as ASCII bytes, for appending to a byte buffer.
    #[inline]
    pub fn format_bytes(&mut self, value: f64) -> &[u8] {
        let bits = value.to_bits();
        let negative = bits >> 63 != 0;
        let exponent = ((bits >> 52) & 0x7ff) as u32;
        let mantissa = bits & ((1 << 52) - 1);
        if exponent == 0x7ff {
            return if mantissa != 0 {
                b"NaN"
            } else if negative {
                b"-inf"
            } else {
                b"inf"
            };
        }
        if exponent == 0 && mantissa == 0 {
            return if negative { b"-0.0" } else { b"0.0" };
        }
        let (digits, e10) = shortest::shortest(exponent, mantissa);
        let (start, end) = self.write(negative, digits, e10);
        &self.bytes[start..end]
    }

    /// Lay out `digits × 10^e10` with its sign; returns the text's range in
    /// `bytes`. The 17 digits are three registers (a first digit and two
    /// words of eight); the point goes into the word it falls in with
    /// shifts, and the text is stored a few words at a time. Nothing just
    /// stored is read back (that would stall on store forwarding).
    #[inline]
    fn write(&mut self, negative: bool, digits: u64, e10: i32) -> (usize, usize) {
        let (first, middle, last, zeros) = digits_17(digits);
        let all = digit_count(digits);
        let count = all - zeros;
        // The first significant digit's index among the 17.
        let lead = 17 - all;
        let scientific = e10 + all as i32 - 1;
        let out = &mut self.bytes;
        let (start, end) = if (0..=15).contains(&scientific) && count > scientific as usize + 1 {
            // 12.34: the point after `whole` digits, at POINT.
            let whole = scientific as usize + 1;
            let at = lead + whole;
            store_with_point(out, POINT - at, first, middle, last, at);
            (POINT - whole, POINT + 1 + count - whole)
        } else if (0..=15).contains(&scientific) {
            // 1200 -> "1200.0": zeros after the digits, then ".0".
            let whole = scientific as usize + 1;
            let block = POINT - lead - whole;
            store_17(out, block, first, middle, last);
            out[block + 17..block + 33].fill(b'0');
            out[POINT..POINT + 2].copy_from_slice(b".0");
            (POINT - whole, POINT + 2)
        } else if (-5..0).contains(&scientific) {
            // 0.00012: "0.", up to four zeros, the digits.
            let zeros = (-scientific - 1) as usize;
            let digits_at = POINT + 1 + zeros;
            out[POINT - 1..POINT + 7].fill(b'0');
            store_17(out, digits_at - lead, first, middle, last);
            out[POINT - 1] = b'0';
            out[POINT] = b'.';
            (POINT - 1, digits_at + count)
        } else {
            // 1.234e+56, 5e-324: the point after the first digit.
            let mut at = if count > 1 {
                store_with_point(out, POINT - lead - 1, first, middle, last, lead + 1);
                POINT + count
            } else {
                store_17(out, POINT - lead - 1, first, middle, last);
                POINT
            };
            out[at] = b'e';
            out[at + 1] = if scientific < 0 { b'-' } else { b'+' };
            at += 2;
            let e = scientific.unsigned_abs();
            if e >= 100 {
                out[at] = b'0' + (e / 100) as u8;
                at += 1;
            }
            if e >= 10 {
                out[at] = b'0' + (e / 10 % 10) as u8;
                at += 1;
            }
            out[at] = b'0' + (e % 10) as u8;
            (POINT - 1, at + 1)
        };
        if negative {
            out[start - 1] = b'-';
            (start - 1, end)
        } else {
            (start, end)
        }
    }
}

/// Store the 17 digits at `block`.
#[inline(always)]
fn store_17(out: &mut [u8; 64], block: usize, first: u8, middle: u64, last: u64) {
    out[block] = first;
    out[block + 1..block + 9].copy_from_slice(&middle.to_le_bytes());
    out[block + 9..block + 17].copy_from_slice(&last.to_le_bytes());
}

/// `word` with a '.' inserted before byte `at` (0..=7); its top byte moves
/// out, and is returned second.
#[inline(always)]
fn insert_point(word: u64, at: usize) -> (u64, u8) {
    let low = word & ((1u64 << (8 * at)) - 1);
    let high = (word >> (8 * at)) << (8 * at) << 8;
    (low | (b'.' as u64) << (8 * at) | high, (word >> 56) as u8)
}

/// Store the 17 digits at `block` with a '.' before digit `at` (1..=16):
/// 18 bytes. The word the point falls in is shifted in registers; the
/// words after it move one byte up.
#[inline(always)]
fn store_with_point(
    out: &mut [u8; 64],
    block: usize,
    first: u8,
    middle: u64,
    last: u64,
    at: usize,
) {
    out[block] = first;
    if at <= 8 {
        let (middle, carry) = insert_point(middle, at - 1);
        out[block + 1..block + 9].copy_from_slice(&middle.to_le_bytes());
        let last_shifted = (last << 8) | carry as u64;
        out[block + 9..block + 17].copy_from_slice(&last_shifted.to_le_bytes());
        out[block + 17] = (last >> 56) as u8;
    } else {
        out[block + 1..block + 9].copy_from_slice(&middle.to_le_bytes());
        let (last, carry) = insert_point(last, at - 9);
        out[block + 9..block + 17].copy_from_slice(&last.to_le_bytes());
        out[block + 17] = carry;
    }
}

/// The number of decimal digits of `value` (1 to 17).
#[inline]
fn digit_count(value: u64) -> usize {
    // floor(bits × log10 2) is the count or one less.
    let bits = 64 - (value | 1).leading_zeros();
    let guess = ((bits * 1233) >> 12) as usize;
    guess + (value >= POW10[guess]) as usize
}

/// The 17 decimal digits of `value` (nonzero, < 10^17, with leading zeros):
/// the first, then two words of eight; and how many are trailing zeros,
/// read off the words (`w ^ "00000000"` has a zero byte for every '0', and
/// the last digit is the highest byte).
#[inline(always)]
fn digits_17(value: u64) -> (u8, u64, u64, usize) {
    const ZEROS: u64 = 0x3030_3030_3030_3030;
    let high = value / 100_000_000; // < 10^9
    let first = b'0' + (high / 100_000_000) as u8;
    let middle = eight_digits((high % 100_000_000) as u32);
    let last = eight_digits((value % 100_000_000) as u32);
    let zeros = if last != ZEROS {
        ((last ^ ZEROS).leading_zeros() / 8) as usize
    } else if middle != ZEROS {
        8 + ((middle ^ ZEROS).leading_zeros() / 8) as usize
    } else {
        16
    };
    (first, middle, last, zeros)
}

/// The eight ASCII digits of `value` (< 10^8) as little-endian bytes, by
/// dividing several digit groups in one multiply ("SWAR"): 10^4 splits it
/// into two 32-bit lanes, 100 each of those into two 16-bit lanes, 10 each
/// of those into two bytes. Each division multiplies by a rounded-up
/// reciprocal, exact for its range (tests/digits.rs checks all of 0..10^8).
#[inline]
#[doc(hidden)]
pub fn eight_digits(value: u32) -> u64 {
    let v = value as u64;
    let high4 = (v * 109_951_163) >> 40; // v / 10^4
    let low4 = v - high4 * 10_000;
    let fours = high4 | low4 << 32;
    let high2 = ((fours * 5243) >> 19) & 0x0000_007f_0000_007f; // each lane / 100
    let low2 = fours - high2 * 100;
    let twos = high2 | low2 << 16;
    let high1 = ((twos * 103) >> 10) & 0x000f_000f_000f_000f; // each lane / 10
    let low1 = twos - high1 * 10;
    (high1 | low1 << 8) | 0x3030_3030_3030_3030
}
