//! The small vector: inline storage that spills to a `Vec`.

use crate::Array;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Bound, Deref, DerefMut, RangeBounds};

/// An index or range past the length; out of line and shared by every
/// element type, so generic code does not format a message per instantiation.
#[cold]
#[inline(never)]
#[track_caller]
fn out_of_bounds(what: &'static str, index: usize, len: usize) -> ! {
    panic!("{what}: index {index} out of bounds (len {len})")
}

/// What a small vector needs of its inline storage (our `ArrayVec`).
#[doc(hidden)]
pub trait Inline<T> {
    type IntoIter: DoubleEndedIterator<Item = T> + ExactSizeIterator;
    fn into_iter(self) -> Self::IntoIter;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    fn try_push(&mut self, value: T) -> Result<(), T>;
    fn pop(&mut self) -> Option<T>;
    fn truncate(&mut self, len: usize);
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
}

impl<T, const N: usize> Inline<T> for arrayvec::ArrayVec<T, N> {
    type IntoIter = arrayvec::IntoIter<T, N>;
    #[inline]
    fn into_iter(self) -> arrayvec::IntoIter<T, N> {
        IntoIterator::into_iter(self)
    }
    #[inline]
    fn len(&self) -> usize {
        arrayvec::ArrayVec::len(self)
    }
    #[inline]
    fn capacity(&self) -> usize {
        N
    }
    #[inline]
    fn try_push(&mut self, value: T) -> Result<(), T> {
        arrayvec::ArrayVec::try_push(self, value).map_err(arrayvec::CapacityError::element)
    }
    #[inline]
    fn pop(&mut self) -> Option<T> {
        arrayvec::ArrayVec::pop(self)
    }
    #[inline]
    fn truncate(&mut self, len: usize) {
        arrayvec::ArrayVec::truncate(self, len)
    }
    #[inline]
    fn as_slice(&self) -> &[T] {
        arrayvec::ArrayVec::as_slice(self)
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        arrayvec::ArrayVec::as_mut_slice(self)
    }
}

enum Data<A: Array> {
    Inline(A::Inline),
    Heap(Vec<A::Item>),
}

/// A vector that keeps up to `A::size()` elements inline.
pub struct SmallVec<A: Array> {
    data: Data<A>,
}

impl<A: Array> SmallVec<A> {
    #[inline]
    pub fn new() -> SmallVec<A> {
        SmallVec {
            data: Data::Inline(A::Inline::default()),
        }
    }

    /// Inline when `capacity` fits, otherwise a `Vec` of that capacity.
    pub fn with_capacity(capacity: usize) -> SmallVec<A> {
        if capacity <= A::size() {
            SmallVec::new()
        } else {
            SmallVec {
                data: Data::Heap(Vec::with_capacity(capacity)),
            }
        }
    }

    /// The elements of `vec`, moved inline when they fit.
    pub fn from_vec(vec: Vec<A::Item>) -> SmallVec<A> {
        let mut out = SmallVec {
            data: Data::Heap(vec),
        };
        out.shrink_to_fit();
        out
    }

    pub fn from_slice(slice: &[A::Item]) -> SmallVec<A>
    where
        A::Item: Clone,
    {
        let mut out = SmallVec::with_capacity(slice.len());
        out.extend_from_slice(slice);
        out
    }

    pub fn from_elem(element: A::Item, n: usize) -> SmallVec<A>
    where
        A::Item: Clone,
    {
        let mut out = SmallVec::with_capacity(n);
        out.resize(n, element);
        out
    }

    /// The inline capacity.
    #[inline]
    pub fn inline_size(&self) -> usize {
        A::size()
    }

    /// Whether the elements live on the heap.
    #[inline]
    pub fn spilled(&self) -> bool {
        matches!(self.data, Data::Heap(_))
    }

    #[inline]
    pub fn len(&self) -> usize {
        match &self.data {
            Data::Inline(inline) => inline.len(),
            Data::Heap(vec) => vec.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        match &self.data {
            Data::Inline(inline) => inline.capacity(),
            Data::Heap(vec) => vec.capacity(),
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[A::Item] {
        match &self.data {
            Data::Inline(inline) => inline.as_slice(),
            Data::Heap(vec) => vec,
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [A::Item] {
        match &mut self.data {
            Data::Inline(inline) => inline.as_mut_slice(),
            Data::Heap(vec) => vec,
        }
    }

    /// The heap vector, spilling the inline elements into one of `capacity`.
    fn heap(&mut self, capacity: usize) -> &mut Vec<A::Item> {
        if let Data::Inline(inline) = &mut self.data {
            let mut vec = Vec::with_capacity(capacity.max(inline.len()));
            let len = inline.len();
            for _ in 0..len {
                vec.push(inline.pop().expect("len"));
            }
            vec.reverse();
            self.data = Data::Heap(vec);
        }
        match &mut self.data {
            Data::Heap(vec) => vec,
            Data::Inline(_) => unreachable!("spilled above"),
        }
    }

    #[inline]
    pub fn push(&mut self, value: A::Item) {
        match &mut self.data {
            Data::Inline(inline) => {
                if let Err(value) = inline.try_push(value) {
                    let doubled = (2 * A::size()).max(1);
                    self.heap(doubled).push(value);
                }
            }
            Data::Heap(vec) => vec.push(value),
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<A::Item> {
        match &mut self.data {
            Data::Inline(inline) => inline.pop(),
            Data::Heap(vec) => vec.pop(),
        }
    }

    /// Insert at `index`, shifting the rest right. Panics when `index > len`.
    #[track_caller]
    pub fn insert(&mut self, index: usize, value: A::Item) {
        let len = self.len();
        if !(index <= len) {
            out_of_bounds("SmallVec::insert", index, len);
        }
        self.push(value);
        self.as_mut_slice()[index..].rotate_right(1);
    }

    /// Insert clones of `slice` at `index`. Panics when `index > len`.
    #[track_caller]
    pub fn insert_from_slice(&mut self, index: usize, slice: &[A::Item])
    where
        A::Item: Clone,
    {
        let len = self.len();
        if !(index <= len) {
            out_of_bounds("SmallVec::insert_from_slice", index, len);
        }
        self.extend_from_slice(slice);
        self.as_mut_slice()[index..].rotate_right(slice.len());
    }

    /// Remove at `index`, shifting the rest left. Panics when out of bounds.
    #[track_caller]
    pub fn remove(&mut self, index: usize) -> A::Item {
        let len = self.len();
        if !(index < len) {
            out_of_bounds("SmallVec::remove", index, len);
        }
        self.as_mut_slice()[index..].rotate_left(1);
        self.pop().expect("in bounds")
    }

    /// Remove at `index`, moving the last element there. Panics when out of bounds.
    #[track_caller]
    pub fn swap_remove(&mut self, index: usize) -> A::Item {
        let len = self.len();
        if !(index < len) {
            out_of_bounds("SmallVec::swap_remove", index, len);
        }
        self.as_mut_slice().swap(index, len - 1);
        self.pop().expect("in bounds")
    }

    pub fn truncate(&mut self, len: usize) {
        match &mut self.data {
            Data::Inline(inline) => inline.truncate(len),
            Data::Heap(vec) => vec.truncate(len),
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Room for `additional` more elements (spilling when inline is too small).
    pub fn reserve(&mut self, additional: usize) {
        let needed = self
            .len()
            .checked_add(additional)
            .expect("SmallVec::reserve: capacity overflow");
        if needed <= self.capacity() {
            return;
        }
        let len = self.len();
        self.heap(needed).reserve(needed - len);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let needed = self
            .len()
            .checked_add(additional)
            .expect("SmallVec::reserve_exact: capacity overflow");
        if needed <= self.capacity() {
            return;
        }
        let len = self.len();
        self.heap(needed).reserve_exact(needed - len);
    }

    /// Move the elements back inline when they fit; otherwise shrink the heap vector.
    pub fn shrink_to_fit(&mut self) {
        let Data::Heap(vec) = &mut self.data else {
            return;
        };
        if vec.len() > A::size() {
            vec.shrink_to_fit();
            return;
        }
        let mut inline = A::Inline::default();
        for value in vec.drain(..) {
            if inline.try_push(value).is_err() {
                unreachable!("fits inline");
            }
        }
        self.data = Data::Inline(inline);
    }

    pub fn extend_from_slice(&mut self, slice: &[A::Item])
    where
        A::Item: Clone,
    {
        self.reserve(slice.len());
        for value in slice {
            self.push(value.clone());
        }
    }

    pub fn resize(&mut self, len: usize, value: A::Item)
    where
        A::Item: Clone,
    {
        let current = self.len();
        if len <= current {
            self.truncate(len);
        } else {
            self.reserve(len - current);
            for _ in current..len {
                self.push(value.clone());
            }
        }
    }

    pub fn resize_with<F: FnMut() -> A::Item>(&mut self, len: usize, mut make: F) {
        let current = self.len();
        if len <= current {
            self.truncate(len);
        } else {
            self.reserve(len - current);
            for _ in current..len {
                self.push(make());
            }
        }
    }

    /// Keep the elements for which `keep` returns true, in order.
    pub fn retain<F: FnMut(&mut A::Item) -> bool>(&mut self, mut keep: F) {
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

    /// The same as [`SmallVec::retain`] (both pass `&mut`).
    pub fn retain_mut<F: FnMut(&mut A::Item) -> bool>(&mut self, keep: F) {
        self.retain(keep);
    }

    /// Remove consecutive repeated elements.
    pub fn dedup(&mut self)
    where
        A::Item: PartialEq,
    {
        self.dedup_by(|a, b| a == b);
    }

    pub fn dedup_by<F: FnMut(&mut A::Item, &mut A::Item) -> bool>(&mut self, mut same: F) {
        let slice = self.as_mut_slice();
        if slice.len() < 2 {
            return;
        }
        let mut kept = 1;
        for i in 1..slice.len() {
            let (front, back) = slice.split_at_mut(i);
            if !same(&mut back[0], &mut front[kept - 1]) {
                slice.swap(kept, i);
                kept += 1;
            }
        }
        self.truncate(kept);
    }

    pub fn dedup_by_key<K: PartialEq, F: FnMut(&mut A::Item) -> K>(&mut self, mut key: F) {
        self.dedup_by(|a, b| key(a) == key(b));
    }

    /// Remove `range` and iterate over its elements.
    #[track_caller]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, A> {
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
            out_of_bounds("SmallVec::drain", end, len);
        }
        self.as_mut_slice()[start..].rotate_left(end - start);
        let mut taken = SmallVec::<A>::with_capacity(end - start);
        for _ in start..end {
            taken.push(self.pop().expect("in range"));
        }
        taken.as_mut_slice().reverse();
        Drain {
            items: taken.into_iter(),
            _vec: core::marker::PhantomData,
        }
    }

    /// The elements as a `Vec` (no copy when spilled).
    pub fn into_vec(self) -> Vec<A::Item> {
        match self.data {
            Data::Heap(vec) => vec,
            Data::Inline(_) => self.into_iter().collect(),
        }
    }

    pub fn into_boxed_slice(self) -> Box<[A::Item]> {
        self.into_vec().into_boxed_slice()
    }

    pub fn to_vec(&self) -> Vec<A::Item>
    where
        A::Item: Clone,
    {
        self.as_slice().to_vec()
    }
}

impl<A: Array> Default for SmallVec<A> {
    fn default() -> SmallVec<A> {
        SmallVec::new()
    }
}

impl<A: Array> Deref for SmallVec<A> {
    type Target = [A::Item];
    #[inline]
    fn deref(&self) -> &[A::Item] {
        self.as_slice()
    }
}

impl<A: Array> DerefMut for SmallVec<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [A::Item] {
        self.as_mut_slice()
    }
}

impl<A: Array> AsRef<[A::Item]> for SmallVec<A> {
    fn as_ref(&self) -> &[A::Item] {
        self
    }
}

impl<A: Array> AsMut<[A::Item]> for SmallVec<A> {
    fn as_mut(&mut self) -> &mut [A::Item] {
        self
    }
}

impl<A: Array> core::borrow::Borrow<[A::Item]> for SmallVec<A> {
    fn borrow(&self) -> &[A::Item] {
        self
    }
}

impl<A: Array> core::borrow::BorrowMut<[A::Item]> for SmallVec<A> {
    fn borrow_mut(&mut self) -> &mut [A::Item] {
        self
    }
}

impl<A: Array> Clone for SmallVec<A>
where
    A::Item: Clone,
{
    fn clone(&self) -> SmallVec<A> {
        SmallVec::from_slice(self)
    }
}

impl<A: Array> fmt::Debug for SmallVec<A>
where
    A::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

impl<A: Array, B: Array> PartialEq<SmallVec<B>> for SmallVec<A>
where
    A::Item: PartialEq<B::Item>,
{
    fn eq(&self, other: &SmallVec<B>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<A: Array> Eq for SmallVec<A> where A::Item: Eq {}

impl<A: Array> PartialOrd for SmallVec<A>
where
    A::Item: PartialOrd,
{
    fn partial_cmp(&self, other: &SmallVec<A>) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<A: Array> Ord for SmallVec<A>
where
    A::Item: Ord,
{
    fn cmp(&self, other: &SmallVec<A>) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<A: Array> Hash for SmallVec<A>
where
    A::Item: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

impl<A: Array> From<Vec<A::Item>> for SmallVec<A> {
    fn from(vec: Vec<A::Item>) -> SmallVec<A> {
        SmallVec::from_vec(vec)
    }
}

impl<T: Clone, A: Array<Item = T>> From<&[T]> for SmallVec<A> {
    fn from(slice: &[T]) -> SmallVec<A> {
        SmallVec::from_slice(slice)
    }
}

impl<T, const N: usize> From<[T; N]> for SmallVec<[T; N]> {
    fn from(array: [T; N]) -> SmallVec<[T; N]> {
        array.into_iter().collect()
    }
}

impl<A: Array> FromIterator<A::Item> for SmallVec<A> {
    fn from_iter<I: IntoIterator<Item = A::Item>>(iter: I) -> SmallVec<A> {
        let mut out = SmallVec::new();
        out.extend(iter);
        out
    }
}

impl<A: Array> Extend<A::Item> for SmallVec<A> {
    fn extend<I: IntoIterator<Item = A::Item>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for value in iter {
            self.push(value);
        }
    }
}

impl<'a, A: Array> IntoIterator for &'a SmallVec<A> {
    type Item = &'a A::Item;
    type IntoIter = core::slice::Iter<'a, A::Item>;
    fn into_iter(self) -> core::slice::Iter<'a, A::Item> {
        self.iter()
    }
}

impl<'a, A: Array> IntoIterator for &'a mut SmallVec<A> {
    type Item = &'a mut A::Item;
    type IntoIter = core::slice::IterMut<'a, A::Item>;
    fn into_iter(self) -> core::slice::IterMut<'a, A::Item> {
        self.iter_mut()
    }
}

impl<A: Array> IntoIterator for SmallVec<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<A>;
    fn into_iter(self) -> IntoIter<A> {
        IntoIter {
            data: match self.data {
                Data::Inline(inline) => Elements::Inline(inline.into_iter()),
                Data::Heap(vec) => Elements::Heap(vec.into_iter()),
            },
        }
    }
}

enum Elements<A: Array> {
    Inline(<A::Inline as Inline<A::Item>>::IntoIter),
    Heap(std::vec::IntoIter<A::Item>),
}

/// The owning iterator of a [`SmallVec`].
pub struct IntoIter<A: Array> {
    data: Elements<A>,
}

impl<A: Array> Iterator for IntoIter<A> {
    type Item = A::Item;
    #[inline]
    fn next(&mut self) -> Option<A::Item> {
        match &mut self.data {
            Elements::Inline(inline) => inline.next(),
            Elements::Heap(heap) => heap.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = match &self.data {
            Elements::Inline(inline) => inline.len(),
            Elements::Heap(heap) => heap.len(),
        };
        (n, Some(n))
    }
}

impl<A: Array> DoubleEndedIterator for IntoIter<A> {
    #[inline]
    fn next_back(&mut self) -> Option<A::Item> {
        match &mut self.data {
            Elements::Inline(inline) => inline.next_back(),
            Elements::Heap(heap) => heap.next_back(),
        }
    }
}

impl<A: Array> ExactSizeIterator for IntoIter<A> {}

/// The elements of a range removed by [`SmallVec::drain`].
pub struct Drain<'a, A: Array> {
    items: IntoIter<A>,
    _vec: core::marker::PhantomData<&'a mut SmallVec<A>>,
}

impl<A: Array> Iterator for Drain<'_, A> {
    type Item = A::Item;
    #[inline]
    fn next(&mut self) -> Option<A::Item> {
        self.items.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}

impl<A: Array> DoubleEndedIterator for Drain<'_, A> {
    fn next_back(&mut self) -> Option<A::Item> {
        self.items.next_back()
    }
}

impl<A: Array> ExactSizeIterator for Drain<'_, A> {}
