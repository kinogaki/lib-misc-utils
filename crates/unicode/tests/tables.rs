//! The generated tables against the UCD files they came from, for every code
//! point (so a stale or hand-edited src/tables.rs fails here).

use std::collections::HashMap;

/// Property value per code point, from a UCD file's `range ; value` lines.
fn parse(file: &str, field: usize, keep: impl Fn(&[&str]) -> bool) -> HashMap<u32, String> {
    let mut out = HashMap::new();
    for line in file.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(';').map(str::trim).collect();
        if !keep(&fields) {
            continue;
        }
        let (a, b) = fields[0].split_once("..").unwrap_or((fields[0], fields[0]));
        for cp in u32::from_str_radix(a, 16).unwrap()..=u32::from_str_radix(b, 16).unwrap() {
            out.insert(cp, fields[field].to_string());
        }
    }
    out
}

#[test]
fn combining_marks_match_the_general_category() {
    let category = parse(include_str!("../ucd/DerivedGeneralCategory.txt"), 1, |_| {
        true
    });
    let mut marks = 0;
    for cp in 0..=0x10ffff {
        let Some(c) = char::from_u32(cp) else {
            continue;
        };
        let expected = matches!(
            category.get(&cp).map(String::as_str),
            Some("Mn" | "Mc" | "Me")
        );
        assert_eq!(kinogaki_unicode::is_mark(c), expected, "U+{cp:04X}");
        marks += expected as u32;
    }
    assert!(marks > 2500, "{marks} marks");
}

/// Each grapheme property, observed through segmentation of two-character
/// strings whose boundary only that property decides.
#[test]
fn grapheme_properties_match_the_ucd() {
    let gcb = parse(include_str!("../ucd/GraphemeBreakProperty.txt"), 1, |_| {
        true
    });
    let pictographic = parse(include_str!("../ucd/emoji-data.txt"), 1, |f| {
        f[1] == "Extended_Pictographic"
    });
    let breaks = |s: &str| kinogaki_unicode::graphemes(s).count() > 1;
    for cp in 0..=0x10ffff {
        let Some(c) = char::from_u32(cp) else {
            continue;
        };
        let value = gcb.get(&cp).map_or("Other", String::as_str);
        // "a" then c joins exactly for Extend, ZWJ and SpacingMark (GB9, GB9a).
        let joins_after_letter = !breaks(&format!("a{c}"));
        assert_eq!(
            joins_after_letter,
            matches!(value, "Extend" | "ZWJ" | "SpacingMark"),
            "U+{cp:04X} {value}"
        );
        // c then "a" joins exactly for Prepend (GB9b).
        let joins_before_letter = !breaks(&format!("{c}a"));
        assert_eq!(
            joins_before_letter,
            value == "Prepend",
            "U+{cp:04X} {value}"
        );
        // ExtPict ZWJ c joins exactly when c is Extended_Pictographic (GB11),
        // or when c joins anything (GB9).
        let after_zwj = !breaks(&format!("\u{1F600}\u{200D}{c}"));
        let expected =
            pictographic.contains_key(&cp) || matches!(value, "Extend" | "ZWJ" | "SpacingMark");
        assert_eq!(after_zwj, expected, "U+{cp:04X} pictographic");
    }
}

#[test]
fn word_properties_match_the_ucd() {
    let wb = parse(include_str!("../ucd/WordBreakProperty.txt"), 1, |_| true);
    let bounds = |s: &str| kinogaki_unicode::word_bound_indices(s).count();
    for cp in 0..=0x10ffff {
        let Some(c) = char::from_u32(cp) else {
            continue;
        };
        let value = wb.get(&cp).map_or("Other", String::as_str);
        // c between two letters: one word for letters (WB5) and for mid-letter
        // punctuation (WB6/WB7) and joiners (WB4, WB13a/b); otherwise three
        // pieces (or two when c also starts or ends a newline).
        let joined = bounds(&format!("a{c}a")) == 1;
        let expected = matches!(
            value,
            "ALetter"
                | "Hebrew_Letter"
                | "MidLetter"
                | "MidNumLet"
                | "Single_Quote"
                | "Numeric"
                | "ExtendNumLet"
                | "Extend"
                | "Format"
                | "ZWJ"
        );
        assert_eq!(joined, expected, "U+{cp:04X} {value}");
        // Between digits: digits (WB8), numeric punctuation (WB11/12), joiners.
        let numeric = bounds(&format!("1{c}1")) == 1;
        let expected = matches!(
            value,
            "Numeric"
                | "ALetter"
                | "Hebrew_Letter"
                | "MidNum"
                | "MidNumLet"
                | "Single_Quote"
                | "ExtendNumLet"
                | "Extend"
                | "Format"
                | "ZWJ"
        );
        assert_eq!(numeric, expected, "U+{cp:04X} {value} between digits");
    }
}
