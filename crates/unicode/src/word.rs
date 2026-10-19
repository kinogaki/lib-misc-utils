//! Word boundaries (UAX #29 §4.1.1, rules WB3-WB999).

use crate::{grapheme, tables};

/// Word_Break values, in the tables' order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Break {
    Other,
    Cr,
    Lf,
    Newline,
    Extend,
    Zwj,
    RegionalIndicator,
    Format,
    Katakana,
    HebrewLetter,
    ALetter,
    SingleQuote,
    DoubleQuote,
    MidNumLet,
    MidLetter,
    MidNum,
    Numeric,
    ExtendNumLet,
    WSegSpace,
}

const BREAKS: [Break; 19] = [
    Break::Other,
    Break::Cr,
    Break::Lf,
    Break::Newline,
    Break::Extend,
    Break::Zwj,
    Break::RegionalIndicator,
    Break::Format,
    Break::Katakana,
    Break::HebrewLetter,
    Break::ALetter,
    Break::SingleQuote,
    Break::DoubleQuote,
    Break::MidNumLet,
    Break::MidLetter,
    Break::MidNum,
    Break::Numeric,
    Break::ExtendNumLet,
    Break::WSegSpace,
];

#[inline]
fn word_break(c: char) -> Break {
    BREAKS[crate::value(tables::WORD, &tables::WORD_ASCII, c) as usize]
}

fn ah_letter(b: Break) -> bool {
    matches!(b, Break::ALetter | Break::HebrewLetter)
}

fn mid_num_let_q(b: Break) -> bool {
    matches!(b, Break::MidNumLet | Break::SingleQuote)
}

fn newline(b: Break) -> bool {
    matches!(b, Break::Newline | Break::Cr | Break::Lf)
}

fn ignorable(b: Break) -> bool {
    matches!(b, Break::Extend | Break::Format | Break::Zwj)
}

/// One character of the text being split.
struct Char {
    at: usize,
    wb: Break,
    pictographic: bool,
    /// WB4: the character whose property this one takes (itself unless it is
    /// an Extend/Format/ZWJ attached to the character before).
    owner: usize,
    /// WB15/16: Regional_Indicators in a row (WB4 applied) ending here.
    regional_run: u32,
}

/// `text` split at every word boundary, with byte offsets: words, spaces and
/// punctuation alike, as UAX #29 defines them.
pub fn word_bound_indices(text: &str) -> WordBoundIndices<'_> {
    let mut chars: Vec<Char> = Vec::with_capacity(text.len());
    for (at, c) in text.char_indices() {
        let wb = word_break(c);
        let i = chars.len();
        let owner = match chars.last() {
            Some(last) if ignorable(wb) && !newline(last.wb) => last.owner,
            _ => i,
        };
        let regional_run = if owner != i {
            chars[owner].regional_run
        } else if wb == Break::RegionalIndicator {
            1 + i
                .checked_sub(1)
                .map_or(0, |p| chars[chars[p].owner].regional_run)
        } else {
            0
        };
        chars.push(Char {
            at,
            wb,
            pictographic: grapheme::props(c).pictographic,
            owner,
            regional_run,
        });
    }
    let mut starts = vec![0];
    for i in 1..chars.len() {
        if breaks_before(&chars, i) {
            starts.push(chars[i].at);
        }
    }
    WordBoundIndices {
        text,
        starts: starts.into_iter().peekable(),
    }
}

/// Whether there is a word boundary before character `i` (0 < i < len).
fn breaks_before(chars: &[Char], i: usize) -> bool {
    use Break::*;
    let previous = &chars[i - 1];
    let next = &chars[i];
    match (previous.wb, next.wb) {
        (Cr, Lf) => return false,                                       // WB3
        (Newline | Cr | Lf, _) | (_, Newline | Cr | Lf) => return true, // WB3a, WB3b
        (Zwj, _) if next.pictographic => return false,                  // WB3c
        (WSegSpace, WSegSpace) => return false,                         // WB3d
        _ => {}
    }
    if next.owner != i {
        return false; // WB4: attached to the character before
    }
    // The properties around the boundary with WB4 applied: p2 p1 | n1 n2.
    let p1_at = previous.owner;
    let p1 = chars[p1_at].wb;
    let p2 = p1_at.checked_sub(1).map(|j| chars[chars[j].owner].wb);
    let n1 = next.wb;
    let n2 = chars[i + 1..]
        .iter()
        .enumerate()
        .find(|(k, c)| c.owner == i + 1 + k)
        .map(|(_, c)| c.wb);
    let ah = ah_letter;
    let is = |b: Option<Break>, f: fn(Break) -> bool| b.is_some_and(f);
    !(ah(p1) && ah(n1)                                                         // WB5
        || ah(p1) && (n1 == MidLetter || mid_num_let_q(n1)) && is(n2, ah_letter) // WB6
        || is(p2, ah_letter) && (p1 == MidLetter || mid_num_let_q(p1)) && ah(n1) // WB7
        || p1 == HebrewLetter && n1 == SingleQuote                                 // WB7a
        || p1 == HebrewLetter && n1 == DoubleQuote && n2 == Some(HebrewLetter)     // WB7b
        || p2 == Some(HebrewLetter) && p1 == DoubleQuote && n1 == HebrewLetter     // WB7c
        || p1 == Numeric && n1 == Numeric                                          // WB8
        || ah(p1) && n1 == Numeric                                                 // WB9
        || p1 == Numeric && ah(n1)                                                 // WB10
        || p2 == Some(Numeric) && (p1 == MidNum || mid_num_let_q(p1)) && n1 == Numeric // WB11
        || p1 == Numeric && (n1 == MidNum || mid_num_let_q(n1)) && n2 == Some(Numeric) // WB12
        || p1 == Katakana && n1 == Katakana                                        // WB13
        || (ah(p1) || matches!(p1, Numeric | Katakana | ExtendNumLet)) && n1 == ExtendNumLet // WB13a
        || p1 == ExtendNumLet && (ah(n1) || matches!(n1, Numeric | Katakana))      // WB13b
        || p1 == RegionalIndicator && n1 == RegionalIndicator && chars[p1_at].regional_run % 2 == 1) // WB15, WB16
}

/// See [`word_bound_indices`].
pub struct WordBoundIndices<'a> {
    text: &'a str,
    starts: core::iter::Peekable<std::vec::IntoIter<usize>>,
}

impl<'a> Iterator for WordBoundIndices<'a> {
    type Item = (usize, &'a str);
    fn next(&mut self) -> Option<(usize, &'a str)> {
        let start = self.starts.next()?;
        if start == self.text.len() {
            return None;
        }
        let end = self.starts.peek().copied().unwrap_or(self.text.len());
        Some((start, &self.text[start..end]))
    }
}
