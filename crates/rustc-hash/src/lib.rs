//! Kinogaki's own `rustc-hash`, published under the crates.io name so the
//! frozen naga, naga-types and wgpu-core resolve to it through
//! `[patch.crates-io]`. It is not upstream rustc-hash: it is the 1.1 FxHasher
//! (rustc's fast, non-cryptographic word hash), producing the same hashes bit
//! for bit (tests/known_hashes.rs), with its map and set aliases.

use core::hash::{BuildHasherDefault, Hasher};

/// A `HashMap` with [`FxHasher`].
pub type FxHashMap<K, V> = std::collections::HashMap<K, V, FxBuildHasher>;
/// A `HashSet` with [`FxHasher`].
pub type FxHashSet<V> = std::collections::HashSet<V, FxBuildHasher>;
/// Builds [`FxHasher`]s (all start from zero).
pub type FxBuildHasher = BuildHasherDefault<FxHasher>;

#[cfg(target_pointer_width = "64")]
const K: usize = 0x517c_c1b7_2722_0a95;
#[cfg(target_pointer_width = "32")]
const K: usize = 0x9e37_79b9;

/// Each word: rotate the state left 5, xor the word in, multiply by `K`.
#[derive(Default, Clone, Copy)]
pub struct FxHasher {
    hash: usize,
}

impl FxHasher {
    #[inline]
    fn add_to_hash(&mut self, word: usize) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(K);
    }
}

const WORD: usize = core::mem::size_of::<usize>();

impl Hasher for FxHasher {
    /// Whole native-endian words, then a 4-, 2- and 1-byte tail as words.
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let (words, mut rest) = bytes.as_chunks::<WORD>();
        for word in words {
            self.add_to_hash(usize::from_ne_bytes(*word));
        }
        if WORD > 4 && rest.len() >= 4 {
            self.add_to_hash(u32::from_ne_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize);
            rest = &rest[4..];
        }
        if WORD > 2 && rest.len() >= 2 {
            self.add_to_hash(u16::from_ne_bytes([rest[0], rest[1]]) as usize);
            rest = &rest[2..];
        }
        if WORD > 1 && !rest.is_empty() {
            self.add_to_hash(rest[0] as usize);
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add_to_hash(i as usize);
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as usize);
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as usize);
        self.add_to_hash((i >> 32) as usize);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add_to_hash(i);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash as u64
    }
}
