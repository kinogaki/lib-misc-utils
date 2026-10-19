//! Kinogaki's own `bit-set`, published under the crates.io name so the frozen
//! naga and wgpu-core resolve to it through `[patch.crates-io]`. It is not
//! upstream bit-set: it has the API those crates call.
//!
//! [`BitSet<B>`] is a set of small `usize` values stored as a [`BitVec<B>`]:
//! value `v` is present when bit `v` is set. Inserting grows the vector;
//! removing never shrinks it, so two sets are equal when they hold the same
//! values whatever their vectors' lengths.

use bit_vec::{BitBlock, BitVec};
use core::fmt;
use core::hash::{Hash, Hasher};

/// A set of small integers as a bit vector.
#[derive(Clone)]
pub struct BitSet<B: BitBlock = u32> {
    bit_vec: BitVec<B>,
}

impl<B: BitBlock> Default for BitSet<B> {
    fn default() -> BitSet<B> {
        BitSet {
            bit_vec: BitVec::default(),
        }
    }
}

impl BitSet<u32> {
    pub fn new() -> BitSet<u32> {
        BitSet::default()
    }

    /// Room for values below `nbits` without growing (the set stays empty).
    pub fn with_capacity(nbits: usize) -> BitSet<u32> {
        BitSet {
            bit_vec: BitVec::from_elem(nbits, false),
        }
    }
}

impl<B: BitBlock> BitSet<B> {
    /// The set of `v` with bit `v` set.
    pub fn from_bit_vec(bit_vec: BitVec<B>) -> BitSet<B> {
        BitSet { bit_vec }
    }

    pub fn into_bit_vec(self) -> BitVec<B> {
        self.bit_vec
    }

    pub fn get_ref(&self) -> &BitVec<B> {
        &self.bit_vec
    }

    pub fn get_mut(&mut self) -> &mut BitVec<B> {
        &mut self.bit_vec
    }

    /// The values below which no insert needs to grow.
    pub fn capacity(&self) -> usize {
        self.bit_vec.capacity()
    }

    /// Reserve room for values below `len`.
    pub fn reserve_len(&mut self, len: usize) {
        let current = self.bit_vec.len();
        if len >= current {
            self.bit_vec.reserve(len - current);
        }
    }

    pub fn reserve_len_exact(&mut self, len: usize) {
        let current = self.bit_vec.len();
        if len >= current {
            self.bit_vec.reserve_exact(len - current);
        }
    }

    pub fn shrink_to_fit(&mut self) {
        let keep = self.iter().last().map_or(0, |v| v + 1);
        self.bit_vec.truncate(keep);
        self.bit_vec.shrink_to_fit();
    }

    /// Add `value`; whether it was absent.
    pub fn insert(&mut self, value: usize) -> bool {
        if self.contains(value) {
            return false;
        }
        let len = self.bit_vec.len();
        if value >= len {
            self.bit_vec.grow(value - len + 1, false);
        }
        self.bit_vec.set(value, true);
        true
    }

    /// Remove `value`; whether it was present.
    pub fn remove(&mut self, value: usize) -> bool {
        if !self.contains(value) {
            return false;
        }
        self.bit_vec.set(value, false);
        true
    }

    #[inline]
    pub fn contains(&self, value: usize) -> bool {
        self.bit_vec.get(value) == Some(true)
    }

    /// The number of values.
    pub fn len(&self) -> usize {
        self.bit_vec.count_ones() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.bit_vec.none()
    }

    /// Remove every value (the capacity stays).
    pub fn make_empty(&mut self) {
        self.bit_vec.fill(false);
    }

    pub fn clear(&mut self) {
        self.make_empty();
    }

    /// The values in ascending order.
    pub fn iter(&self) -> Iter<'_, B> {
        Iter {
            blocks: self.bit_vec.blocks().enumerate(),
            current: None,
        }
    }

    pub fn union_with(&mut self, other: &BitSet<B>) {
        self.match_len(other);
        let mut other = other.bit_vec.clone();
        other.grow(self.bit_vec.len() - other.len(), false);
        self.bit_vec.or(&other);
    }

    pub fn intersect_with(&mut self, other: &BitSet<B>) {
        let mut other = other.bit_vec.clone();
        Self::fit(&mut other, self.bit_vec.len());
        self.bit_vec.and(&other);
    }

    pub fn difference_with(&mut self, other: &BitSet<B>) {
        let mut other = other.bit_vec.clone();
        Self::fit(&mut other, self.bit_vec.len());
        self.bit_vec.difference(&other);
    }

    pub fn symmetric_difference_with(&mut self, other: &BitSet<B>) {
        self.match_len(other);
        let mut other = other.bit_vec.clone();
        other.grow(self.bit_vec.len() - other.len(), false);
        self.bit_vec.xor(&other);
    }

    pub fn is_subset(&self, other: &BitSet<B>) -> bool {
        self.iter().all(|v| other.contains(v))
    }

    pub fn is_superset(&self, other: &BitSet<B>) -> bool {
        other.is_subset(self)
    }

    pub fn is_disjoint(&self, other: &BitSet<B>) -> bool {
        self.iter().all(|v| !other.contains(v))
    }

    fn match_len(&mut self, other: &BitSet<B>) {
        let (mine, theirs) = (self.bit_vec.len(), other.bit_vec.len());
        if theirs > mine {
            self.bit_vec.grow(theirs - mine, false);
        }
    }

    /// `vec` at exactly `len` bits (growing with zeros or truncating).
    fn fit(vec: &mut BitVec<B>, len: usize) {
        if vec.len() < len {
            vec.grow(len - vec.len(), false);
        } else {
            vec.truncate(len);
        }
    }
}

impl<B: BitBlock> PartialEq for BitSet<B> {
    fn eq(&self, other: &BitSet<B>) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<B: BitBlock> Eq for BitSet<B> {}

impl<B: BitBlock> Hash for BitSet<B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in self {
            value.hash(state);
        }
    }
}

impl<B: BitBlock> fmt::Debug for BitSet<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<B: BitBlock> Extend<usize> for BitSet<B> {
    fn extend<I: IntoIterator<Item = usize>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<B: BitBlock> FromIterator<usize> for BitSet<B> {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> BitSet<B> {
        let mut out = BitSet::default();
        out.extend(iter);
        out
    }
}

impl<'a, B: BitBlock> IntoIterator for &'a BitSet<B> {
    type Item = usize;
    type IntoIter = Iter<'a, B>;
    fn into_iter(self) -> Iter<'a, B> {
        self.iter()
    }
}

/// The values of a [`BitSet`], ascending; whole zero blocks are skipped.
#[derive(Clone)]
pub struct Iter<'a, B: BitBlock> {
    blocks: core::iter::Enumerate<bit_vec::Blocks<'a, B>>,
    /// The block being read: its index and the bits not yet yielded.
    current: Option<(usize, B)>,
}

impl<B: BitBlock> Iterator for Iter<'_, B> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        loop {
            if let Some((index, bits)) = &mut self.current
                && *bits != B::zero()
            {
                // The lowest set bit: count the zeros below it.
                let mut position = 0;
                while (*bits >> position) & B::one() == B::zero() {
                    position += 1;
                }
                *bits = *bits & !(B::one() << position);
                return Some(*index * B::bits() + position);
            }
            self.current = Some(self.blocks.next()?);
        }
    }
}
