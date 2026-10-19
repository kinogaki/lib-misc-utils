//! Fast search for one byte value in a byte slice, in safe code: [`find`],
//! [`rfind`] and [`count`].
//!
//! The slice is read eight bytes at a time as a `u64` ("SWAR"). For a word
//! `w` and the needle repeated in every byte, `x = w ^ needle` has a zero
//! byte exactly where `w` holds the needle, and
//! `!(((x & 0x7f..7f) + 0x7f..7f) | x) & 0x80..80` sets the top bit of
//! exactly those bytes (no carry crosses a byte, unlike the shorter
//! `(x - 0x01..01) & !x` test). The lowest or highest such bit gives the
//! first or last match; the bit count gives the number of matches. Blocks of
//! four words are tested together before one is looked at.

const LOW7: u64 = 0x7f7f_7f7f_7f7f_7f7f;
const HIGH: u64 = 0x8080_8080_8080_8080;
const ONES: u64 = 0x0101_0101_0101_0101;

/// The top bit of every byte of `x` that is zero.
#[inline(always)]
fn zero_bytes(x: u64) -> u64 {
    !(((x & LOW7).wrapping_add(LOW7)) | x) & HIGH
}

#[inline(always)]
fn word(bytes: &[u8]) -> u64 {
    let mut w = [0; 8];
    w.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(w)
}

/// The index of the first `needle` in `haystack`.
#[inline]
pub fn find(needle: u8, haystack: &[u8]) -> Option<usize> {
    let pattern = ONES * needle as u64;
    let mut at = 0;
    // Short slices (OBJ lines are) go straight to the word loop.
    while at + 32 <= haystack.len() {
        let block = &haystack[at..at + 32];
        let m = [
            zero_bytes(word(&block[0..]) ^ pattern),
            zero_bytes(word(&block[8..]) ^ pattern),
            zero_bytes(word(&block[16..]) ^ pattern),
            zero_bytes(word(&block[24..]) ^ pattern),
        ];
        if m[0] | m[1] | m[2] | m[3] != 0 {
            let (i, bits) = m
                .iter()
                .enumerate()
                .find(|(_, b)| **b != 0)
                .expect("a match");
            return Some(at + 8 * i + (bits.trailing_zeros() / 8) as usize);
        }
        at += 32;
    }
    while at + 8 <= haystack.len() {
        let m = zero_bytes(word(&haystack[at..]) ^ pattern);
        if m != 0 {
            return Some(at + (m.trailing_zeros() / 8) as usize);
        }
        at += 8;
    }
    haystack[at..]
        .iter()
        .position(|&b| b == needle)
        .map(|i| at + i)
}

/// The index of the last `needle` in `haystack`.
#[inline]
pub fn rfind(needle: u8, haystack: &[u8]) -> Option<usize> {
    let pattern = ONES * needle as u64;
    let mut end = haystack.len();
    while end >= 8 {
        let m = zero_bytes(word(&haystack[end - 8..]) ^ pattern);
        if m != 0 {
            return Some(end - 8 + (7 - m.leading_zeros() / 8) as usize);
        }
        end -= 8;
    }
    haystack[..end].iter().rposition(|&b| b == needle)
}

/// How many times `needle` occurs in `haystack`.
#[inline]
pub fn count(needle: u8, haystack: &[u8]) -> usize {
    let pattern = ONES * needle as u64;
    let (words, rest) = haystack.as_chunks::<8>();
    let mut total = 0;
    for w in words {
        total += zero_bytes(u64::from_le_bytes(*w) ^ pattern).count_ones() as usize;
    }
    total + rest.iter().filter(|&&b| b == needle).count()
}
