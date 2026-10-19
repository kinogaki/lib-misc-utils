//! The UCD's GraphemeBreakTest.txt and WordBreakTest.txt: every line is a
//! string of code points with `÷` (boundary) or `×` (none) between them.

/// (text, byte offsets of every boundary inside or at the end of it).
fn cases(file: &str) -> Vec<(String, Vec<usize>)> {
    let mut out = Vec::new();
    for line in file.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let mut text = String::new();
        let mut boundaries = Vec::new();
        for token in line.split_whitespace() {
            match token {
                "÷" => boundaries.push(text.len()),
                "×" => {}
                hex => text.push(char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap()),
            }
        }
        boundaries.retain(|&b| b != 0);
        out.push((text, boundaries));
    }
    out
}

fn ends<'a>(pieces: impl Iterator<Item = (usize, &'a str)>) -> Vec<usize> {
    pieces.map(|(at, s)| at + s.len()).collect()
}

#[test]
fn grapheme_break_test() {
    let cases = cases(include_str!("../ucd/GraphemeBreakTest.txt"));
    assert!(cases.len() > 700);
    for (text, expected) in &cases {
        assert_eq!(
            &ends(kinogaki_unicode::grapheme_indices(text)),
            expected,
            "{:x?}",
            text.chars().map(|c| c as u32).collect::<Vec<_>>()
        );
        let joined: String = kinogaki_unicode::graphemes(text).collect();
        assert_eq!(&joined, text);
    }
}

#[test]
fn word_break_test() {
    let cases = cases(include_str!("../ucd/WordBreakTest.txt"));
    assert!(cases.len() > 1800);
    for (text, expected) in &cases {
        assert_eq!(
            &ends(kinogaki_unicode::word_bound_indices(text)),
            expected,
            "{:x?}",
            text.chars().map(|c| c as u32).collect::<Vec<_>>()
        );
    }
}

#[test]
fn everyday_text() {
    let words: Vec<&str> = kinogaki_unicode::word_bound_indices(
        "The quick (\"brown\") fox can't jump 32.3 feet, right?",
    )
    .map(|(_, w)| w)
    .collect();
    assert_eq!(
        words,
        [
            "The", " ", "quick", " ", "(", "\"", "brown", "\"", ")", " ", "fox", " ", "can't", " ",
            "jump", " ", "32.3", " ", "feet", ",", " ", "right", "?"
        ]
    );
    let clusters: Vec<&str> = kinogaki_unicode::graphemes("e\u{301}🇯🇵👨‍👩‍👧a\r\n").collect();
    assert_eq!(clusters, ["e\u{301}", "🇯🇵", "👨‍👩‍👧", "a", "\r\n"]);
    assert_eq!(kinogaki_unicode::graphemes("").count(), 0);
    assert_eq!(kinogaki_unicode::word_bound_indices("").count(), 0);
}
