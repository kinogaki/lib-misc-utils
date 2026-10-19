//! Hashes recorded from rustc-hash 1.1.0 on a 64-bit little-endian target
//! (`str` hashing writes the bytes, then 0xff). Ours must match bit for bit.

use rustc_hash::{FxHashMap, FxHasher};
use std::hash::{Hash, Hasher};

fn fx<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
#[test]
fn matches_rustc_hash_1_1() {
    let known = [
        ("", 0x2b44f56ffae88a6b),
        ("a", 0xaa44c3c5b8e22aff),
        ("ab", 0x3326e72fb14cbfa7),
        ("abc", 0x32061e1b06958893),
        ("abcd", 0x65fab557a3117e21),
        ("abcde", 0xdff717ada948ea37),
        ("abcdefg", 0x8d58cb512476ad60),
        ("abcdefgh", 0xdc23285da99c6aff),
        ("abcdefghi", 0x5d5247d4d678b8f9),
        ("kinogaki studio", 0x6efe28214c37d3fe),
        ("the quick brown fox jumps", 0xdc7cfd85652f966b),
    ];
    for (text, hash) in known {
        assert_eq!(fx(text), hash, "{text:?}");
    }
    assert_eq!(fx(&(1u32, 2u8, 3u16, 4u64, 5usize)), 0xb41eb5bc5ba67cc7);
}

#[test]
fn maps_work() {
    let mut map: FxHashMap<&str, u32> = FxHashMap::default();
    map.insert("a", 1);
    *map.entry("a").or_default() += 1;
    assert_eq!(map["a"], 2);
}
