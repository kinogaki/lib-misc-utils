//! Kinogaki's own `foldhash`, published under the crates.io name so the
//! frozen hashbrown resolves to it through `[patch.crates-io]` for its default
//! hasher. It is not upstream foldhash: it is our own fast, seeded,
//! non-cryptographic hash with the `fast::RandomState` hashbrown builds.
//!
//! Each 64-bit word is mixed with a *folded multiply* (the 128-bit product of
//! two words, high half xor low half), keyed by the builder's random seeds;
//! the result's high bits, which hashbrown's control bytes use, depend on
//! every input bit. Seeds are random per process and differ per builder, as
//! upstream's, so map iteration order is not stable between runs.

pub mod fast {
    pub use crate::{FoldHasher, RandomState};
}

use core::hash::{BuildHasher, Hasher};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// The 128-bit product of `a` and `b`, high half xor low half.
#[inline(always)]
const fn folded_multiply(a: u64, b: u64) -> u64 {
    let full = (a as u128).wrapping_mul(b as u128);
    (full as u64) ^ ((full >> 64) as u64)
}

/// Digits of π, as fixed odd keys mixed with the random seeds.
const ARBITRARY: [u64; 3] = [
    0x243f_6a88_85a3_08d3,
    0x1319_8a2e_0370_7345,
    0xa409_3822_299f_31d1,
];

/// Builds [`FoldHasher`]s with this builder's seeds.
#[derive(Clone, Debug)]
pub struct RandomState {
    key: u64,
    finish_key: u64,
}

impl RandomState {
    pub fn new() -> RandomState {
        // A random per-process base, stepped per builder by an odd constant.
        static BASE: OnceLock<u64> = OnceLock::new();
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let base = *BASE.get_or_init(|| {
            std::hash::BuildHasher::hash_one(&std::hash::RandomState::new(), 0x5eed_u64)
        });
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let seed = folded_multiply(
            base ^ ARBITRARY[0],
            n.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1,
        );
        RandomState {
            key: seed | 1,
            finish_key: folded_multiply(seed, ARBITRARY[1]) | 1,
        }
    }
}

impl Default for RandomState {
    fn default() -> RandomState {
        RandomState::new()
    }
}

impl BuildHasher for RandomState {
    type Hasher = FoldHasher;
    #[inline]
    fn build_hasher(&self) -> FoldHasher {
        FoldHasher {
            accumulator: self.key ^ ARBITRARY[2],
            key: self.key,
            finish_key: self.finish_key,
        }
    }
}

/// The hasher: one folded multiply per 64-bit word written.
#[derive(Clone, Debug)]
pub struct FoldHasher {
    accumulator: u64,
    key: u64,
    finish_key: u64,
}

impl FoldHasher {
    #[inline(always)]
    fn mix(&mut self, word: u64) {
        self.accumulator = folded_multiply(self.accumulator ^ word, self.key ^ ARBITRARY[0]);
    }
}

impl Hasher for FoldHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let (words, rest) = bytes.as_chunks::<8>();
        for word in words {
            self.mix(u64::from_le_bytes(*word));
        }
        // The tail in the low bytes and its length in the top byte, so
        // "a" and "a\0" differ.
        let mut word = [0; 8];
        word[..rest.len()].copy_from_slice(rest);
        word[7] = rest.len() as u8;
        self.mix(u64::from_le_bytes(word));
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.mix(i as u64);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.mix(i as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.mix(i as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.mix(i);
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.mix(i as u64);
        self.mix((i >> 64) as u64);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.mix(i as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        folded_multiply(self.accumulator, self.finish_key)
    }
}
