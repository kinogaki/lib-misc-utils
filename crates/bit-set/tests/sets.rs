use bit_set::BitSet;
use bit_vec::BitVec;

#[test]
fn bit_vec_basics() {
    let mut v = BitVec::from_elem(70, false);
    v.set(3, true);
    v.set(69, true);
    assert_eq!(v.get(3), Some(true));
    assert_eq!(v.get(70), None);
    assert!(v[69] && !v[68]);
    assert_eq!(v.count_ones(), 2);
    v.grow(10, true);
    assert_eq!(v.len(), 80);
    assert_eq!(v.count_ones(), 12);
    v.truncate(65);
    assert_eq!(v.count_ones(), 1);
    assert_eq!(v.blocks().count(), 3);
    v.clear();
    assert!(v.none() && v.len() == 65, "clear keeps the length");
    v.fill(true);
    assert_eq!(v.count_ones(), 65, "bits past len stay zero");
    assert!(v.all());
    assert_eq!(v.pop(), Some(true));
    let mut w: BitVec<usize> = BitVec::default();
    w.grow(130, false);
    w.set(129, true);
    assert_eq!(w.blocks().next_back(), Some(2));
    let bytes = BitVec::from_bytes(&[0b1010_0000]);
    assert_eq!(format!("{bytes:?}"), "10100000");
    let mut a = BitVec::from_bytes(&[0b1100_0000]);
    assert!(a.or(&BitVec::from_bytes(&[0b0010_0000])));
    assert!(!a.or(&BitVec::from_bytes(&[0b0010_0000])));
    assert!(a.difference(&BitVec::from_bytes(&[0b1000_0000])));
    assert_eq!(a, BitVec::from_bytes(&[0b0110_0000]));
    // naga's HandleSet::pop: clear the highest set bit through iter_mut.
    let popped = a.iter_mut().enumerate().rev().find_map(|(i, mut bit)| {
        if *bit {
            *bit = false;
            Some(i)
        } else {
            None
        }
    });
    assert_eq!(popped, Some(2));
    assert_eq!(a, BitVec::from_bytes(&[0b0100_0000]));
    *a.get_mut(7).unwrap() = true;
    assert_eq!(a, BitVec::from_bytes(&[0b0100_0001]));
}

#[test]
fn bit_set_basics() {
    let mut s = BitSet::new();
    assert!(s.insert(5));
    assert!(!s.insert(5));
    assert!(s.insert(100));
    assert!(s.insert(31) && s.insert(32) && s.insert(0));
    assert_eq!(s.iter().collect::<Vec<_>>(), [0, 5, 31, 32, 100]);
    assert_eq!(s.len(), 5);
    assert!(s.remove(5) && !s.remove(5));
    assert!(!s.contains(5) && s.contains(100) && !s.contains(10_000));
    let other: BitSet = [31, 7, 200].into_iter().collect();
    let mut u = s.clone();
    u.union_with(&other);
    assert_eq!(u.iter().collect::<Vec<_>>(), [0, 7, 31, 32, 100, 200]);
    let mut i = s.clone();
    i.intersect_with(&other);
    assert_eq!(i.iter().collect::<Vec<_>>(), [31]);
    let mut d = s.clone();
    d.difference_with(&other);
    assert_eq!(d.iter().collect::<Vec<_>>(), [0, 32, 100]);
    let mut x = s.clone();
    x.symmetric_difference_with(&other);
    assert_eq!(x.iter().collect::<Vec<_>>(), [0, 7, 32, 100, 200]);
    assert!(i.is_subset(&s) && s.is_superset(&i) && d.is_disjoint(&other));
    let mut grown = BitSet::with_capacity(1000);
    grown.insert(31);
    assert_eq!(grown, i, "equality ignores capacity");
    grown.make_empty();
    assert!(grown.is_empty() && grown.get_ref().len() == 1000);
    let back = BitSet::from_bit_vec(s.clone().into_bit_vec());
    assert_eq!(back, s);
    assert_eq!(format!("{i:?}"), "{31}");
}
