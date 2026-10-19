//! SmallVec behaviour inline and spilled, with drops counted.

use smallvec::{SmallVec, smallvec};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct Counted(u32, Rc<Cell<u32>>);

impl Drop for Counted {
    fn drop(&mut self) {
        self.1.set(self.1.get() + 1);
    }
}

#[test]
fn spills_past_the_inline_capacity() {
    let mut v: SmallVec<[u32; 2]> = SmallVec::new();
    assert_eq!(v.inline_size(), 2);
    v.push(1);
    v.push(2);
    assert!(!v.spilled());
    v.push(3);
    assert!(v.spilled());
    assert_eq!(&v[..], &[1, 2, 3]);
    assert!(v.capacity() >= 4);
    v.truncate(1);
    assert!(v.spilled(), "stays on the heap");
    v.shrink_to_fit();
    assert!(!v.spilled());
    assert_eq!(&v[..], &[1]);
    assert!(SmallVec::<[u32; 2]>::with_capacity(5).spilled());
    assert!(!SmallVec::<[u32; 2]>::from_vec(vec![1]).spilled());
}

#[test]
fn editing_inline_and_on_the_heap() {
    for spill in [false, true] {
        let mut v: SmallVec<[i32; 8]> = smallvec![1, 2, 3, 4];
        if spill {
            v.reserve(20);
            assert!(v.spilled());
        }
        v.insert(1, 9);
        assert_eq!(&v[..], &[1, 9, 2, 3, 4]);
        assert_eq!(v.remove(0), 1);
        assert_eq!(v.swap_remove(0), 9);
        assert_eq!(&v[..], &[4, 2, 3]);
        v.insert_from_slice(1, &[7, 8]);
        assert_eq!(&v[..], &[4, 7, 8, 2, 3]);
        v.retain(|x| *x != 8);
        assert_eq!(&v[..], &[4, 7, 2, 3]);
        assert_eq!(v.drain(1..3).collect::<Vec<_>>(), [7, 2]);
        assert_eq!(&v[..], &[4, 3]);
        v.resize(4, 0);
        v.extend_from_slice(&[5, 5, 6]);
        v.dedup();
        assert_eq!(&v[..], &[4, 3, 0, 5, 6]);
        assert_eq!(v.clone().into_vec(), vec![4, 3, 0, 5, 6]);
        assert_eq!(v.into_iter().rev().collect::<Vec<_>>(), [6, 5, 0, 3, 4]);
    }
}

#[test]
fn every_element_drops_once() {
    let drops = Rc::new(Cell::new(0));
    {
        let mut v: SmallVec<[Counted; 2]> = SmallVec::new();
        for i in 0..5 {
            v.push(Counted(i, drops.clone()));
        }
        assert_eq!(v.remove(1).0, 1);
        let mut it = v.drain(..2);
        drop(it.next());
        drop(it);
        let mut rest = v.into_iter();
        drop(rest.next());
    }
    assert_eq!(drops.get(), 5);
    let mut v: SmallVec<[Counted; 4]> = (0..3).map(|i| Counted(i, drops.clone())).collect();
    v.clear();
    assert_eq!(drops.get(), 8);
}

#[test]
fn macros_and_comparisons() {
    let a: SmallVec<[u8; 4]> = smallvec![7; 3];
    let b: SmallVec<[u8; 1]> = smallvec![7, 7, 7];
    assert_eq!(a, b);
    assert_eq!(format!("{a:?}"), "[7, 7, 7]");
    let from_array = SmallVec::from([1, 2]);
    assert_eq!(from_array.len(), 2);
    let empty: SmallVec<[u8; 0]> = smallvec![];
    assert!(empty.is_empty());
}
