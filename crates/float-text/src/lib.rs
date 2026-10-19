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
//! Ryu and zmij do), and the text is zmij's byte for byte. The digits come
//! from one 128-bit scaling (src/shortest.rs), written eight at a time.
//! The text is plain decimal when the value's scientific exponent `E` is in
//! -5..=15 (`123.456`, `0.00001`, always with a `.`: `1.0`), and `d.ddde±E`
//! outside (`1.5e+16`, `1e-6`).

mod digits;
mod shortest;
mod tables;

#[doc(hidden)]
pub use digits::sixteen_digits;

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

/// Where the first significant digit goes in `bytes` (after "0." and zeros
/// for small values): room for a sign and a block's leading zeros before it,
/// 17 digits, the point and an exponent after.
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
        let (integral, last, e10) = shortest::shortest(exponent, mantissa);
        let (start, end) = self.write(negative, integral, last, e10);
        &self.bytes[start..end]
    }

    /// Lay out `integral × 10^e10` (15 or 16 digits; subnormals fewer)
    /// followed by the ASCII digit `last` unless it is 0, with its sign;
    /// returns the text's range in `bytes`. The digits are stored as one
    /// 16-digit block (leading zeros fall before the text) and the text is
    /// built around it with fixed-size stores and moves (no `memmove` calls).
    #[inline]
    fn write(&mut self, negative: bool, integral: u64, last: u8, e10: i32) -> (usize, usize) {
        const ZEROS: u64 = 0x3030_3030_3030_3030;
        let out = &mut self.bytes;
        let (high, low) = digits::sixteen_digits(
            (integral / 100_000_000) as u32,
            (integral % 100_000_000) as u32,
        );
        // Normal values have 15 or 16 digits here; subnormals may have fewer.
        let all = if integral >= 100_000_000_000_000 {
            15 + (integral >= 1_000_000_000_000_000) as usize
        } else {
            digit_count(integral)
        };
        // Trailing '0's of the block: `w ^ "00000000"` has a zero byte for
        // each, and the last digit is the highest byte.
        let zeros = ((low ^ ZEROS).leading_zeros() / 8) as usize
            + (low == ZEROS) as usize * ((high ^ ZEROS).leading_zeros() / 8) as usize;
        let count = if last != 0 {
            all + 1
        } else {
            all - zeros.min(all - 1)
        };
        let scientific = e10 + all as i32 - 1;
        // The 16 digits as one number, leading zeros first (bytes 0..16 - all).
        let block = high as u128 | (low as u128) << 64;
        let (start, end) = if (0..=15).contains(&scientific) && count > scientific as usize + 1 {
            // 12.34: the digits, then the fraction one place right (a
            // fixed 16-byte move, which compiles to one load and store).
            let whole = scientific as usize + 1;
            let first = POINT - (16 - all);
            out[first..first + 16].copy_from_slice(&block.to_le_bytes());
            out[POINT + all] = last;
            out.copy_within(POINT + whole..POINT + whole + 16, POINT + whole + 1);
            out[POINT + whole] = b'.';
            (POINT, POINT + count + 1)
        } else if (0..=15).contains(&scientific) {
            // 1200 -> "1200.0": the digits, zeros, ".0".
            let first = POINT - (16 - all);
            out[first..first + 16].copy_from_slice(&block.to_le_bytes());
            out[POINT + all] = last;
            out[POINT + count..POINT + count + 16].fill(b'0');
            let whole = scientific as usize + 1;
            out[POINT + whole..POINT + whole + 2].copy_from_slice(b".0");
            (POINT, POINT + whole + 2)
        } else if (-5..0).contains(&scientific) {
            // 0.00012: zeros first, the digits after "0." and the zeros,
            // then "0." over the block's leading zeros.
            let zeros = (-scientific - 1) as usize;
            let digits_at = POINT + 1 + zeros;
            out[POINT - 1..POINT + 7].fill(b'0');
            let first = digits_at - (16 - all);
            out[first..first + 16].copy_from_slice(&block.to_le_bytes());
            out[digits_at + all] = last;
            out[POINT - 1] = b'0';
            out[POINT] = b'.';
            (POINT - 1, digits_at + count)
        } else {
            // 1.234e+56: the first digit (from the registers, not read
            // back) before the point, then "e±N" from the table in one word.
            // (A tiny subnormal's integral part is 0: its one digit is `last`.)
            let first = POINT - (16 - all);
            out[first..first + 16].copy_from_slice(&block.to_le_bytes());
            out[POINT + all] = last;
            out[POINT - 1] = if all == 0 {
                last
            } else {
                (block >> (8 * (16 - all))) as u8
            };
            out[POINT] = b'.';
            let at = POINT + count - (count == 1) as usize;
            let (text, len) = EXPONENTS[(scientific - EXPONENT_MIN) as usize];
            out[at..at + 8].copy_from_slice(&text.to_le_bytes());
            (POINT - 1, at + len as usize)
        };
        out[start - 1] = b'-';
        (start - negative as usize, end)
    }
}

/// The number of decimal digits of `value` (nonzero, below 10^17).
#[inline]
fn digit_count(value: u64) -> usize {
    // floor(bits × log10 2) is the count or one less.
    let bits = 64 - (value | 1).leading_zeros();
    let guess = ((bits * 1233) >> 12) as usize;
    guess + (value >= POW10[guess]) as usize
}

/// The smallest exponent [`EXPONENTS`] spells.
const EXPONENT_MIN: i32 = -340;

/// "e+N" / "e-N" for each exponent from -340 to 340, as little-endian bytes
/// with the length.
static EXPONENTS: [(u64, u8); 681] = exponents();

const fn exponents() -> [(u64, u8); 681] {
    let mut table = [(0u64, 0u8); 681];
    let mut i = 0;
    while i < 681 {
        let e = EXPONENT_MIN + i as i32;
        let magnitude = e.unsigned_abs() as u64;
        let sign = if e < 0 { b'-' } else { b'+' };
        let mut bytes = b'e' as u64 | (sign as u64) << 8;
        let mut len = 2;
        if magnitude >= 100 {
            bytes |= (b'0' as u64 + magnitude / 100) << (8 * len);
            len += 1;
        }
        if magnitude >= 10 {
            bytes |= (b'0' as u64 + magnitude / 10 % 10) << (8 * len);
            len += 1;
        }
        bytes |= (b'0' as u64 + magnitude % 10) << (8 * len);
        len += 1;
        table[i] = (bytes, len);
        i += 1;
    }
    table
}
