//! The fixed-capacity vector, in safe code over [`Buffer`].

use crate::buffer::{Buffer, Drainer};
use crate::{CapacityError, fail};
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{Bound, Deref, DerefMut, RangeBounds};

/// A vector with inline storage for at most `CAP` elements.
pub struct ArrayVec<T, const CAP: usize> {
    buffer: Buffer<T, CAP>,
}

impl<T, const CAP: usize> ArrayVec<T, CAP> {
    /// The capacity, `CAP`.
    pub const CAPACITY: usize = CAP;

    #[inline]
    pub const fn new() -> ArrayVec<T, CAP> {
        ArrayVec {
            buffer: Buffer::new(),
        }
    }

    #[inline]
    pub const fn new_const() -> ArrayVec<T, CAP> {
        ArrayVec::new()
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len() == CAP
    }

    #[inline]
    pub const fn remaining_capacity(&self) -> usize {
        CAP - self.len()
    }

    /// Append `element`. Panics when full.
    #[inline]
    #[track_caller]
    pub fn push(&mut self, element: T) {
        if self.buffer.push(element).is_err() {
            fail::capacity_exceeded("ArrayVec::push");
        }
    }

    /// Append `element`, or return it in the error when full.
    #[inline]
    pub fn try_push(&mut self, element: T) -> Result<(), CapacityError<T>> {
        self.buffer.push(element).map_err(CapacityError::new)
    }

    /// Insert `element` at `index`, shifting the rest right. Panics when
    /// `index > len` or when full.
    #[track_caller]
    pub fn insert(&mut self, index: usize, element: T) {
        if self.try_insert(index, element).is_err() {
            fail::capacity_exceeded("ArrayVec::insert");
        }
    }

    /// Insert `element` at `index`, or return it when full. Panics when `index > len`.
    #[track_caller]
    pub fn try_insert(&mut self, index: usize, element: T) -> Result<(), CapacityError<T>> {
        let len = self.len();
        if index > len {
            fail::out_of_bounds("ArrayVec::insert", index, len);
        }
        self.try_push(element)?;
        self.as_mut_slice()[index..].rotate_right(1);
        Ok(())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.buffer.pop()
    }

    /// Remove and return the element at `index`, shifting the rest left.
    /// Panics when out of bounds.
    #[track_caller]
    pub fn remove(&mut self, index: usize) -> T {
        let len = self.len();
        match self.pop_at(index) {
            Some(element) => element,
            None => fail::out_of_bounds("ArrayVec::remove", index, len),
        }
    }

    /// [`ArrayVec::remove`], `None` when out of bounds.
    pub fn pop_at(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        self.as_mut_slice()[index..].rotate_left(1);
        self.pop()
    }

    /// Remove the element at `index`, moving the last one into its place.
    /// Panics when out of bounds.
    #[track_caller]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let len = self.len();
        match self.swap_pop(index) {
            Some(element) => element,
            None => fail::out_of_bounds("ArrayVec::swap_remove", index, len),
        }
    }

    /// [`ArrayVec::swap_remove`], `None` when out of bounds.
    pub fn swap_pop(&mut self, index: usize) -> Option<T> {
        let len = self.len();
        if index >= len {
            return None;
        }
        self.as_mut_slice().swap(index, len - 1);
        self.pop()
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.buffer.truncate(len);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.buffer.truncate(0);
    }

    /// Keep the elements for which `keep` returns true, in order.
    pub fn retain<F: FnMut(&mut T) -> bool>(&mut self, mut keep: F) {
        let slice = self.as_mut_slice();
        let mut kept = 0;
        for i in 0..slice.len() {
            if keep(&mut slice[i]) {
                slice.swap(kept, i);
                kept += 1;
            }
        }
        self.truncate(kept);
    }

    /// Remove `range` and iterate over its elements.
    #[track_caller]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, T, CAP> {
        let len = self.len();
        let start = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&e) => e + 1,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => len,
        };
        if start > end || end > len {
            fail::out_of_bounds("ArrayVec::drain", end, len);
        }
        // Move the range behind the tail, then take it off the end: the
        // buffer then holds it last-first, which `Drain` pops front-first.
        self.as_mut_slice()[start..].rotate_left(end - start);
        let mut items = Buffer::new();
        for _ in start..end {
            let item = self.pop().expect("in range");
            if items.push(item).is_err() {
                unreachable!("a drained range fits the capacity it came from");
            }
        }
        Drain {
            items: items.into_drainer(),
            _vec: PhantomData,
        }
    }

    /// Clone and append every element of `other`, or fail without changing
    /// anything when they do not all fit.
    pub fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        if self.remaining_capacity() < other.len() {
            return Err(CapacityError::new(()));
        }
        for item in other {
            self.push(item.clone());
        }
        Ok(())
    }

    /// The array, when full; otherwise `self` back.
    pub fn into_inner(self) -> Result<[T; CAP], ArrayVec<T, CAP>> {
        if !self.is_full() {
            return Err(self);
        }
        let mut items = self.into_iter();
        Ok(core::array::from_fn(|_| items.next().expect("full")))
    }

    /// The contents, leaving `self` empty.
    pub fn take(&mut self) -> ArrayVec<T, CAP> {
        core::mem::take(self)
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.buffer.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.buffer.as_mut_slice()
    }
}

impl<T, const CAP: usize> Default for ArrayVec<T, CAP> {
    #[inline]
    fn default() -> ArrayVec<T, CAP> {
        ArrayVec::new()
    }
}

impl<T, const CAP: usize> Deref for ArrayVec<T, CAP> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const CAP: usize> DerefMut for ArrayVec<T, CAP> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const CAP: usize> AsRef<[T]> for ArrayVec<T, CAP> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const CAP: usize> AsMut<[T]> for ArrayVec<T, CAP> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const CAP: usize> Borrow<[T]> for ArrayVec<T, CAP> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T, const CAP: usize> BorrowMut<[T]> for ArrayVec<T, CAP> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: Clone, const CAP: usize> Clone for ArrayVec<T, CAP> {
    fn clone(&self) -> ArrayVec<T, CAP> {
        self.iter().cloned().collect()
    }
}

impl<T: fmt::Debug, const CAP: usize> fmt::Debug for ArrayVec<T, CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

impl<T: PartialEq, const CAP: usize> PartialEq for ArrayVec<T, CAP> {
    fn eq(&self, other: &ArrayVec<T, CAP>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialEq, const CAP: usize> PartialEq<[T]> for ArrayVec<T, CAP> {
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<T: Eq, const CAP: usize> Eq for ArrayVec<T, CAP> {}

impl<T: PartialOrd, const CAP: usize> PartialOrd for ArrayVec<T, CAP> {
    fn partial_cmp(&self, other: &ArrayVec<T, CAP>) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: Ord, const CAP: usize> Ord for ArrayVec<T, CAP> {
    fn cmp(&self, other: &ArrayVec<T, CAP>) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T: Hash, const CAP: usize> Hash for ArrayVec<T, CAP> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self.as_slice(), state)
    }
}

impl<T, const CAP: usize> From<[T; CAP]> for ArrayVec<T, CAP> {
    fn from(array: [T; CAP]) -> ArrayVec<T, CAP> {
        array.into_iter().collect()
    }
}

impl<T: Clone, const CAP: usize> TryFrom<&[T]> for ArrayVec<T, CAP> {
    type Error = CapacityError;
    fn try_from(slice: &[T]) -> Result<ArrayVec<T, CAP>, CapacityError> {
        let mut out = ArrayVec::new();
        out.try_extend_from_slice(slice)?;
        Ok(out)
    }
}

/// Panics when the iterator yields more than `CAP` elements.
impl<T, const CAP: usize> FromIterator<T> for ArrayVec<T, CAP> {
    #[track_caller]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> ArrayVec<T, CAP> {
        let mut out = ArrayVec::new();
        out.extend(iter);
        out
    }
}

/// Panics when the elements do not fit.
impl<T, const CAP: usize> Extend<T> for ArrayVec<T, CAP> {
    #[track_caller]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<'a, T: Copy + 'a, const CAP: usize> Extend<&'a T> for ArrayVec<T, CAP> {
    #[track_caller]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        for item in iter {
            self.push(*item);
        }
    }
}

impl<'a, T, const CAP: usize> IntoIterator for &'a ArrayVec<T, CAP> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;
    fn into_iter(self) -> core::slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T, const CAP: usize> IntoIterator for &'a mut ArrayVec<T, CAP> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;
    fn into_iter(self) -> core::slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T, const CAP: usize> IntoIterator for ArrayVec<T, CAP> {
    type Item = T;
    type IntoIter = IntoIter<T, CAP>;
    fn into_iter(self) -> IntoIter<T, CAP> {
        IntoIter {
            items: self.buffer.into_drainer(),
        }
    }
}

/// The owning iterator of an [`ArrayVec`].
pub struct IntoIter<T, const CAP: usize> {
    items: Drainer<T, CAP>,
}

impl<T, const CAP: usize> IntoIter<T, CAP> {
    /// The elements not yet yielded.
    pub fn as_slice(&self) -> &[T] {
        self.items.as_slice()
    }
}

impl<T, const CAP: usize> Iterator for IntoIter<T, CAP> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<T> {
        self.items.front()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.items.len(), Some(self.items.len()))
    }
}

impl<T, const CAP: usize> DoubleEndedIterator for IntoIter<T, CAP> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.items.back()
    }
}

impl<T, const CAP: usize> ExactSizeIterator for IntoIter<T, CAP> {}

impl<T: fmt::Debug, const CAP: usize> fmt::Debug for IntoIter<T, CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

/// The elements of a range removed by [`ArrayVec::drain`].
pub struct Drain<'a, T, const CAP: usize> {
    /// The drained elements, last first (so `next` takes from the back).
    items: Drainer<T, CAP>,
    _vec: PhantomData<&'a mut ArrayVec<T, CAP>>,
}

impl<T, const CAP: usize> Iterator for Drain<'_, T, CAP> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<T> {
        self.items.back()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.items.len(), Some(self.items.len()))
    }
}

impl<T, const CAP: usize> DoubleEndedIterator for Drain<'_, T, CAP> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.items.front()
    }
}

impl<T, const CAP: usize> ExactSizeIterator for Drain<'_, T, CAP> {}
