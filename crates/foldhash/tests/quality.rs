//! The hash spreads its input over every output bit, which hashbrown's
//! control bytes (the top 7 bits) and bucket index (the low bits) need.

use core::hash::{BuildHasher, Hasher};
use foldhash::fast::RandomState;

#[test]
fn every_input_bit_flips_about_half_the_output() {
    let state = RandomState::new();
    let base = |x: u64| state.hash_one(x);
    let mut total = 0u64;
    let mut worst_bit_count = [0u32; 64];
    let samples = 2000u64;
    for i in 0..samples {
        let x = i.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        for bit in 0..64 {
            let diff = base(x) ^ base(x ^ (1 << bit));
            total += diff.count_ones() as u64;
            for (out, count) in worst_bit_count.iter_mut().enumerate() {
                *count += ((diff >> out) & 1) as u32;
            }
        }
    }
    let average = total as f64 / (samples * 64) as f64;
    assert!(
        (30.0..34.0).contains(&average),
        "average flipped bits {average}"
    );
    for (out, count) in worst_bit_count.iter().enumerate() {
        let rate = *count as f64 / (samples * 64) as f64;
        assert!(
            (0.45..0.55).contains(&rate),
            "output bit {out} flips {rate}"
        );
    }
}

#[test]
fn sequential_keys_fill_buckets_and_control_bytes_evenly() {
    let state = RandomState::new();
    let mut low = [0u32; 256];
    let mut top = [0u32; 128];
    for i in 0..256_000u64 {
        let h = state.hash_one(i);
        low[(h & 255) as usize] += 1;
        top[(h >> 57) as usize] += 1;
    }
    assert!(low.iter().all(|&n| (800..1200).contains(&n)), "{low:?}");
    assert!(top.iter().all(|&n| (1700..2300).contains(&n)), "{top:?}");
}

#[test]
fn builders_differ_and_byte_tails_count() {
    let a = RandomState::new();
    let b = RandomState::new();
    assert_ne!(a.hash_one("kinogaki"), b.hash_one("kinogaki"));
    assert_eq!(a.hash_one("kinogaki"), a.clone().hash_one("kinogaki"));
    let bytes = |s: &[u8]| {
        let mut h = a.build_hasher();
        h.write(s);
        h.finish()
    };
    assert_ne!(bytes(b"a"), bytes(b"a\0"));
    assert_ne!(bytes(b""), bytes(b"\0"));
    assert_ne!(bytes(b"12345678"), bytes(b"12345678\0"));
    let mut map: std::collections::HashMap<u64, u64, RandomState> = Default::default();
    for i in 0..1000 {
        map.insert(i, i * 2);
    }
    assert_eq!(map[&999], 1998);
}
