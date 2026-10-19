//! Every function against the obvious byte loop, at every length and
//! alignment and for needles that stress the word trick (0x00, 0x80, 0xff).

fn check(haystack: &[u8], needle: u8) {
    assert_eq!(
        kinogaki_bytes::find(needle, haystack),
        haystack.iter().position(|&b| b == needle),
        "find {needle:#x} in {haystack:?}"
    );
    assert_eq!(
        kinogaki_bytes::rfind(needle, haystack),
        haystack.iter().rposition(|&b| b == needle),
        "rfind {needle:#x} in {haystack:?}"
    );
    assert_eq!(
        kinogaki_bytes::count(needle, haystack),
        haystack.iter().filter(|&&b| b == needle).count(),
        "count {needle:#x}"
    );
}

#[test]
fn every_position_and_length() {
    for len in 0..80 {
        for needle in [b'\n', 0x00, 0x80, 0xff, 0x01, 0x7f] {
            // Background bytes next to the needle in value: borrows and carries.
            for background in [
                needle.wrapping_add(1),
                needle.wrapping_sub(1),
                needle ^ 0x80,
                0x00,
                0xff,
            ] {
                if background == needle {
                    continue;
                }
                let mut haystack = vec![background; len];
                check(&haystack, needle);
                for i in 0..len {
                    haystack[i] = needle;
                    check(&haystack, needle);
                    for j in i + 1..len.min(i + 10) {
                        haystack[j] = needle;
                        check(&haystack, needle);
                        haystack[j] = background;
                    }
                    haystack[i] = background;
                }
            }
        }
    }
}

#[test]
fn random_text() {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let bytes: Vec<u8> = (0..20_000)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            if state.is_multiple_of(23) {
                b'\n'
            } else {
                (state >> 32) as u8
            }
        })
        .collect();
    for start in 0..40 {
        for end in [start, start + 1, start + 31, start + 33, 5000, 20_000] {
            let slice = &bytes[start..end.max(start)];
            for needle in [b'\n', b' ', 0, 0xff] {
                check(slice, needle);
            }
        }
    }
}
