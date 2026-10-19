//! ArrayVec and ArrayString behaviour, with every element's drops counted.

use arrayvec::{ArrayString, ArrayVec};
use std::cell::Cell;
use std::rc::Rc;

/// Counts its drops in a shared cell.
#[derive(Debug)]
struct Counted(u32, Rc<Cell<u32>>);

impl Drop for Counted {
    fn drop(&mut self) {
        self.1.set(self.1.get() + 1);
    }
}

fn counted<const N: usize>(n: u32) -> (ArrayVec<Counted, N>, Rc<Cell<u32>>) {
    let drops = Rc::new(Cell::new(0));
    let v = (0..n).map(|i| Counted(i, drops.clone())).collect();
    (v, drops)
}

fn values<const N: usize>(v: &ArrayVec<Counted, N>) -> Vec<u32> {
    v.iter().map(|c| c.0).collect()
}

#[test]
fn push_pop_and_capacity() {
    let mut v = ArrayVec::<u8, 3>::new();
    assert!(v.is_empty() && v.capacity() == 3 && v.remaining_capacity() == 3);
    v.push(1);
    v.push(2);
    assert_eq!(v.try_push(3), Ok(()));
    assert!(v.is_full());
    assert_eq!(v.try_push(4).unwrap_err().element(), 4);
    assert_eq!(v.pop(), Some(3));
    assert_eq!(&v[..], &[1, 2]);
    assert_eq!(ArrayVec::<u8, 3>::CAPACITY, 3);
}

#[test]
#[should_panic(expected = "ArrayVec::push: capacity exceeded")]
fn push_past_capacity_panics() {
    let mut v = ArrayVec::<u8, 2>::new();
    v.extend([1, 2, 3]);
}

#[test]
fn insert_remove_swap_remove_keep_order() {
    let (mut v, drops) = counted::<8>(5);
    let d = drops.clone();
    v.insert(2, Counted(9, d.clone()));
    assert_eq!(values(&v), [0, 1, 9, 2, 3, 4]);
    let removed = v.remove(0);
    assert_eq!(removed.0, 0);
    drop(removed);
    assert_eq!(values(&v), [1, 9, 2, 3, 4]);
    assert_eq!(v.swap_remove(1).0, 9);
    assert_eq!(values(&v), [1, 4, 2, 3]);
    assert!(v.pop_at(10).is_none());
    assert!(v.try_insert(4, Counted(7, d.clone())).is_ok());
    assert_eq!(values(&v), [1, 4, 2, 3, 7]);
    drop(v);
    assert_eq!(drops.get(), 7, "every element dropped exactly once");
}

#[test]
fn drain_and_retain() {
    let (mut v, drops) = counted::<8>(8);
    let drained: Vec<u32> = v.drain(2..5).map(|c| c.0).collect();
    assert_eq!(drained, [2, 3, 4]);
    assert_eq!(values(&v), [0, 1, 5, 6, 7]);
    assert_eq!(
        v.drain(..).rev().map(|c| c.0).collect::<Vec<_>>(),
        [7, 6, 5, 1, 0]
    );
    assert!(v.is_empty());
    assert_eq!(drops.get(), 8);

    let (mut v, drops) = counted::<8>(8);
    v.retain(|c| c.0 % 3 != 0);
    assert_eq!(values(&v), [1, 2, 4, 5, 7]);
    assert_eq!(drops.get(), 3);
    // A drain dropped half-way drops what it did not yield.
    let mut drain = v.drain(1..4);
    drop(drain.next());
    drop(drain);
    assert_eq!(values(&v), [1, 7]);
    assert_eq!(drops.get(), 6);
    v.truncate(1);
    v.clear();
    assert_eq!(drops.get(), 8);
}

#[test]
fn owning_iteration_drops_the_rest() {
    let (v, drops) = counted::<6>(6);
    let mut it = v.into_iter();
    assert_eq!(it.next().map(|c| c.0), Some(0));
    assert_eq!(it.next_back().map(|c| c.0), Some(5));
    assert_eq!(it.len(), 4);
    drop(it);
    assert_eq!(drops.get(), 6);
}

#[test]
fn conversions() {
    let v = ArrayVec::from([1, 2, 3]);
    assert_eq!(v.clone().into_inner(), Ok([1, 2, 3]));
    let mut short = ArrayVec::<i32, 4>::new();
    short.push(1);
    assert!(short.clone().into_inner().is_err());
    let mut taken = ArrayVec::<i32, 4>::try_from(&[1, 2][..]).unwrap();
    assert!(ArrayVec::<i32, 1>::try_from(&[1, 2][..]).is_err());
    assert_eq!(taken.take().as_slice(), &[1, 2]);
    assert!(taken.is_empty());
    assert!(short.try_extend_from_slice(&[2, 3, 4]).is_ok());
    assert!(short.try_extend_from_slice(&[5]).is_err());
    assert_eq!(short, ArrayVec::from([1, 2, 3, 4]));
    assert_eq!(format!("{short:?}"), "[1, 2, 3, 4]");
    let nested: ArrayVec<ArrayVec<u8, 2>, 2> = [ArrayVec::from([1, 2]), ArrayVec::from([3, 4])]
        .into_iter()
        .collect();
    assert_eq!(nested[1][0], 3);
}

#[test]
fn array_string() {
    let mut s = ArrayString::<8>::new();
    s.push_str("héllo");
    assert_eq!(s.len(), 6);
    assert!(s.try_push_str("abc").is_err());
    assert_eq!(s.as_str(), "héllo");
    s.push('!');
    assert!(s.try_push('é').is_err());
    s.push('a');
    assert!(s.is_full());
    assert_eq!(s.to_string(), "héllo!a");
    use std::fmt::Write;
    let mut hex = ArrayString::<4>::new();
    write!(hex, "{:02x}{:02x}", 0xab, 0x01).unwrap();
    assert_eq!(hex, "ab01");
    assert!(write!(hex, "x").is_err());
}
