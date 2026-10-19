//! Kinogaki's own `bit-vec`, published under the crates.io name so the frozen
//! wgpu-core (and our bit-set) resolve to it through `[patch.crates-io]`. It
//! is not upstream bit-vec: it has the API those crates call.
//!
//! [`BitVec<B>`] stores bits in blocks of `B` (u32 by default), bit `i` at
//! block `i / B::BITS`, position `i % B::BITS`. Invariant: the bits past
//! `len()` in the last block are zero, so blocks compare and count directly.
//! As upstream, `clear` sets every bit to false and keeps the length.

use core::cell::RefCell;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{BitAnd, BitOr, BitXor, Deref, DerefMut, Index, Not, Shl, Shr};
use std::rc::Rc;

/// An unsigned integer used as a block of bits.
pub trait BitBlock:
    Copy
    + Eq
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Hash
    + fmt::Debug
{
    /// Bits per block.
    fn bits() -> usize;
    fn zero() -> Self;
    fn one() -> Self;
    fn count_ones(self) -> usize;
}

macro_rules! bit_block {
    ($($t:ty)*) => {$(
        impl BitBlock for $t {
            #[inline]
            fn bits() -> usize { <$t>::BITS as usize }
            #[inline]
            fn zero() -> $t { 0 }
            #[inline]
            fn one() -> $t { 1 }
            #[inline]
            fn count_ones(self) -> usize { <$t>::count_ones(self) as usize }
        }
    )*};
}
bit_block!(u8 u16 u32 u64 usize);

/// A growable vector of bits.
#[derive(Clone)]
pub struct BitVec<B: BitBlock = u32> {
    storage: Vec<B>,
    nbits: usize,
}

static TRUE: bool = true;
static FALSE: bool = false;

impl<B: BitBlock> Default for BitVec<B> {
    fn default() -> BitVec<B> {
        BitVec {
            storage: Vec::new(),
            nbits: 0,
        }
    }
}

fn blocks_for<B: BitBlock>(bits: usize) -> usize {
    bits.div_ceil(B::bits())
}

impl BitVec<u32> {
    pub fn new() -> BitVec<u32> {
        BitVec::default()
    }

    /// `nbits` bits, all `bit`.
    pub fn from_elem(nbits: usize, bit: bool) -> BitVec<u32> {
        BitVec::from_elem_general(nbits, bit)
    }

    pub fn with_capacity(nbits: usize) -> BitVec<u32> {
        BitVec {
            storage: Vec::with_capacity(blocks_for::<u32>(nbits)),
            nbits: 0,
        }
    }

    /// Bits from bytes, most significant bit of each byte first.
    pub fn from_bytes(bytes: &[u8]) -> BitVec<u32> {
        let mut out = BitVec::with_capacity(bytes.len() * 8);
        for byte in bytes {
            for i in (0..8).rev() {
                out.push(byte >> i & 1 == 1);
            }
        }
        out
    }

    /// `len` bits, bit `i` from `f(i)`.
    pub fn from_fn<F: FnMut(usize) -> bool>(len: usize, mut f: F) -> BitVec<u32> {
        (0..len).map(&mut f).collect()
    }
}

impl<B: BitBlock> BitVec<B> {
    /// `nbits` bits, all `bit`, in blocks of `B`.
    pub fn from_elem_general(nbits: usize, bit: bool) -> BitVec<B> {
        let block = if bit { !B::zero() } else { B::zero() };
        let mut out = BitVec {
            storage: vec![block; blocks_for::<B>(nbits)],
            nbits,
        };
        out.fix_last_block();
        out
    }

    /// Zero the bits past `len()` in the last block.
    fn fix_last_block(&mut self) {
        let extra = self.nbits % B::bits();
        if extra > 0
            && let Some(last) = self.storage.last_mut()
        {
            *last = *last & mask::<B>(extra);
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.nbits
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nbits == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.storage.capacity().saturating_mul(B::bits())
    }

    #[inline]
    pub fn get(&self, i: usize) -> Option<bool> {
        if i >= self.nbits {
            return None;
        }
        let block = self.storage[i / B::bits()];
        Some((block >> (i % B::bits())) & B::one() == B::one())
    }

    /// Set bit `i`. Panics when `i >= len()`.
    #[inline]
    #[track_caller]
    pub fn set(&mut self, i: usize, bit: bool) {
        assert!(
            i < self.nbits,
            "BitVec::set: index {i} out of bounds (len {})",
            self.nbits
        );
        let flag = B::one() << (i % B::bits());
        let block = &mut self.storage[i / B::bits()];
        *block = if bit { *block | flag } else { *block & !flag };
    }

    /// Every bit set to false; the length stays.
    pub fn clear(&mut self) {
        self.fill(false);
    }

    pub fn fill(&mut self, bit: bool) {
        let block = if bit { !B::zero() } else { B::zero() };
        self.storage.fill(block);
        self.fix_last_block();
    }

    pub fn any(&self) -> bool {
        self.storage.iter().any(|&b| b != B::zero())
    }

    pub fn none(&self) -> bool {
        !self.any()
    }

    pub fn all(&self) -> bool {
        (0..self.nbits).all(|i| self.get(i) == Some(true))
    }

    /// The number of bits set.
    pub fn count_ones(&self) -> u64 {
        self.storage.iter().map(|&b| b.count_ones() as u64).sum()
    }

    /// Append `n` bits of `value`.
    pub fn grow(&mut self, n: usize, value: bool) {
        let new_len = self
            .nbits
            .checked_add(n)
            .expect("BitVec::grow: capacity overflow");
        let old_len = self.nbits;
        self.storage.resize(blocks_for::<B>(new_len), B::zero());
        self.nbits = new_len;
        if value {
            for i in old_len..new_len {
                self.set(i, true);
            }
        }
    }

    /// Drop the bits from `len` on.
    pub fn truncate(&mut self, len: usize) {
        if len < self.nbits {
            self.nbits = len;
            self.storage.truncate(blocks_for::<B>(len));
            self.fix_last_block();
        }
    }

    pub fn push(&mut self, bit: bool) {
        if self.nbits.is_multiple_of(B::bits()) {
            self.storage.push(B::zero());
        }
        self.nbits += 1;
        self.set(self.nbits - 1, bit);
    }

    pub fn pop(&mut self) -> Option<bool> {
        let last = self.get(self.nbits.checked_sub(1)?)?;
        self.truncate(self.nbits - 1);
        Some(last)
    }

    /// Room for `additional` more bits.
    pub fn reserve(&mut self, additional: usize) {
        let blocks = blocks_for::<B>(self.nbits.saturating_add(additional));
        self.storage
            .reserve(blocks.saturating_sub(self.storage.len()));
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let blocks = blocks_for::<B>(self.nbits.saturating_add(additional));
        self.storage
            .reserve_exact(blocks.saturating_sub(self.storage.len()));
    }

    pub fn shrink_to_fit(&mut self) {
        self.storage.shrink_to_fit();
    }

    /// The blocks, lowest bits first (bits past `len()` are zero).
    pub fn blocks(&self) -> Blocks<'_, B> {
        Blocks {
            iter: self.storage.iter(),
        }
    }

    pub fn storage(&self) -> &[B] {
        &self.storage
    }

    pub fn iter(&self) -> Iter<'_, B> {
        Iter {
            vec: self,
            front: 0,
            back: self.nbits,
        }
    }

    /// The bits as [`MutBorrowedBit`]s, each written back when dropped.
    pub fn iter_mut(&mut self) -> IterMut<'_, B> {
        let back = self.nbits;
        IterMut {
            vec: Rc::new(RefCell::new(self)),
            front: 0,
            back,
        }
    }

    /// Bit `index` as a [`MutBorrowedBit`], or `None` past the end.
    pub fn get_mut(&mut self, index: usize) -> Option<MutBorrowedBit<'_, B>> {
        let value = self.get(index)?;
        Some(MutBorrowedBit {
            vec: Rc::new(RefCell::new(self)),
            index,
            value,
        })
    }

    /// `self |= other` over the common length; whether anything changed.
    pub fn or(&mut self, other: &BitVec<B>) -> bool {
        self.combine(other, |a, b| a | b)
    }

    /// `self &= other`; whether anything changed.
    pub fn and(&mut self, other: &BitVec<B>) -> bool {
        self.combine(other, |a, b| a & b)
    }

    /// `self &= !other`; whether anything changed.
    pub fn difference(&mut self, other: &BitVec<B>) -> bool {
        self.combine(other, |a, b| a & !b)
    }

    /// `self ^= other`; whether anything changed.
    pub fn xor(&mut self, other: &BitVec<B>) -> bool {
        self.combine(other, |a, b| a ^ b)
    }

    /// Panics unless both have the same length, as upstream.
    #[track_caller]
    fn combine(&mut self, other: &BitVec<B>, f: impl Fn(B, B) -> B) -> bool {
        assert_eq!(self.nbits, other.nbits, "BitVec: lengths differ");
        let mut changed = false;
        for (a, &b) in self.storage.iter_mut().zip(&other.storage) {
            let next = f(*a, b);
            changed |= next != *a;
            *a = next;
        }
        self.fix_last_block();
        changed
    }
}

/// The low `bits` bits set.
fn mask<B: BitBlock>(bits: usize) -> B {
    if bits >= B::bits() {
        !B::zero()
    } else {
        !(!B::zero() << bits)
    }
}

impl<B: BitBlock> Index<usize> for BitVec<B> {
    type Output = bool;
    #[track_caller]
    fn index(&self, i: usize) -> &bool {
        match self.get(i) {
            Some(true) => &TRUE,
            Some(false) => &FALSE,
            None => panic!("BitVec: index {i} out of bounds (len {})", self.nbits),
        }
    }
}

impl<B: BitBlock> PartialEq for BitVec<B> {
    fn eq(&self, other: &BitVec<B>) -> bool {
        self.nbits == other.nbits && self.storage == other.storage
    }
}

impl<B: BitBlock> Eq for BitVec<B> {}

impl<B: BitBlock> Hash for BitVec<B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nbits.hash(state);
        self.storage.hash(state);
    }
}

impl<B: BitBlock> fmt::Debug for BitVec<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bit in self.iter() {
            f.write_str(if bit { "1" } else { "0" })?;
        }
        Ok(())
    }
}

impl<B: BitBlock> Extend<bool> for BitVec<B> {
    fn extend<I: IntoIterator<Item = bool>>(&mut self, iter: I) {
        for bit in iter {
            self.push(bit);
        }
    }
}

impl<B: BitBlock> FromIterator<bool> for BitVec<B> {
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> BitVec<B> {
        let mut out = BitVec::default();
        out.extend(iter);
        out
    }
}

impl<'a, B: BitBlock> IntoIterator for &'a BitVec<B> {
    type Item = bool;
    type IntoIter = Iter<'a, B>;
    fn into_iter(self) -> Iter<'a, B> {
        self.iter()
    }
}

/// The bits of a [`BitVec`].
#[derive(Clone)]
pub struct Iter<'a, B: BitBlock> {
    vec: &'a BitVec<B>,
    front: usize,
    back: usize,
}

impl<B: BitBlock> Iterator for Iter<'_, B> {
    type Item = bool;
    #[inline]
    fn next(&mut self) -> Option<bool> {
        if self.front == self.back {
            return None;
        }
        self.front += 1;
        self.vec.get(self.front - 1)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.back - self.front, Some(self.back - self.front))
    }
}

impl<B: BitBlock> DoubleEndedIterator for Iter<'_, B> {
    fn next_back(&mut self) -> Option<bool> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        self.vec.get(self.back)
    }
}

impl<B: BitBlock> ExactSizeIterator for Iter<'_, B> {}

/// The blocks of a [`BitVec`].
#[derive(Clone)]
pub struct Blocks<'a, B: BitBlock> {
    iter: core::slice::Iter<'a, B>,
}

impl<B: BitBlock> Iterator for Blocks<'_, B> {
    type Item = B;
    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.next().copied()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<B: BitBlock> DoubleEndedIterator for Blocks<'_, B> {
    fn next_back(&mut self) -> Option<B> {
        self.iter.next_back().copied()
    }
}

impl<B: BitBlock> ExactSizeIterator for Blocks<'_, B> {}

/// One bit of a [`BitVec`] as a mutable `bool`; the vector gets the value
/// back when this is dropped.
pub struct MutBorrowedBit<'a, B: BitBlock> {
    vec: Rc<RefCell<&'a mut BitVec<B>>>,
    index: usize,
    value: bool,
}

impl<B: BitBlock> Deref for MutBorrowedBit<'_, B> {
    type Target = bool;
    fn deref(&self) -> &bool {
        &self.value
    }
}

impl<B: BitBlock> DerefMut for MutBorrowedBit<'_, B> {
    fn deref_mut(&mut self) -> &mut bool {
        &mut self.value
    }
}

impl<B: BitBlock> Drop for MutBorrowedBit<'_, B> {
    fn drop(&mut self) {
        self.vec.borrow_mut().set(self.index, self.value);
    }
}

/// The bits of a [`BitVec`] as [`MutBorrowedBit`]s.
pub struct IterMut<'a, B: BitBlock> {
    vec: Rc<RefCell<&'a mut BitVec<B>>>,
    front: usize,
    back: usize,
}

impl<'a, B: BitBlock> IterMut<'a, B> {
    fn bit(&self, index: usize) -> MutBorrowedBit<'a, B> {
        let value = self.vec.borrow().get(index).expect("in range");
        MutBorrowedBit {
            vec: self.vec.clone(),
            index,
            value,
        }
    }
}

impl<'a, B: BitBlock> Iterator for IterMut<'a, B> {
    type Item = MutBorrowedBit<'a, B>;
    fn next(&mut self) -> Option<MutBorrowedBit<'a, B>> {
        if self.front == self.back {
            return None;
        }
        self.front += 1;
        Some(self.bit(self.front - 1))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.back - self.front, Some(self.back - self.front))
    }
}

impl<'a, B: BitBlock> DoubleEndedIterator for IterMut<'a, B> {
    fn next_back(&mut self) -> Option<MutBorrowedBit<'a, B>> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        Some(self.bit(self.back))
    }
}

impl<B: BitBlock> ExactSizeIterator for IterMut<'_, B> {}
