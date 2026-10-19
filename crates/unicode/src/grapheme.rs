//! Extended grapheme clusters (UAX #29 §3.1.1, rules GB3-GB999).

use crate::tables;

/// Grapheme_Cluster_Break values, in the tables' order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(crate) enum Break {
    Other,
    Cr,
    Lf,
    Control,
    Extend,
    Zwj,
    RegionalIndicator,
    Prepend,
    SpacingMark,
    L,
    V,
    T,
    Lv,
    Lvt,
}

/// Indic_Conjunct_Break values, in the tables' order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(crate) enum Conjunct {
    None,
    Consonant,
    Extend,
    Linker,
}

/// The grapheme properties of one character.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Props {
    pub(crate) gcb: Break,
    pub(crate) pictographic: bool,
    pub(crate) conjunct: Conjunct,
}

const BREAKS: [Break; 14] = [
    Break::Other,
    Break::Cr,
    Break::Lf,
    Break::Control,
    Break::Extend,
    Break::Zwj,
    Break::RegionalIndicator,
    Break::Prepend,
    Break::SpacingMark,
    Break::L,
    Break::V,
    Break::T,
    Break::Lv,
    Break::Lvt,
];
const CONJUNCTS: [Conjunct; 4] = [
    Conjunct::None,
    Conjunct::Consonant,
    Conjunct::Extend,
    Conjunct::Linker,
];

#[inline]
pub(crate) fn props(c: char) -> Props {
    let v = crate::value(tables::GRAPHEME, &tables::GRAPHEME_ASCII, c);
    Props {
        gcb: BREAKS[(v & 15) as usize],
        pictographic: v & 16 != 0,
        conjunct: CONJUNCTS[(v >> 5 & 3) as usize],
    }
}

/// What the rules need to know about the cluster so far.
#[derive(Clone, Copy)]
struct State {
    previous: Break,
    /// GB12/13: Regional_Indicators in a row, ending at the previous character.
    regional_run: u32,
    /// GB11: the previous characters are `ExtPict Extend*`.
    pictographic: bool,
    /// GB11: the previous character is the ZWJ after `ExtPict Extend*`.
    zwj_after_pictographic: bool,
    /// GB9c: after an InCB consonant and only InCB Extend/Linker since;
    /// whether a Linker came.
    after_consonant: bool,
    linked: bool,
}

impl State {
    fn new(first: Props) -> State {
        let mut state = State {
            previous: first.gcb,
            regional_run: 0,
            pictographic: false,
            zwj_after_pictographic: false,
            after_consonant: false,
            linked: false,
        };
        state.take(first);
        state
    }

    /// Whether the cluster continues with `next` (no boundary before it).
    fn joins(&self, next: Props) -> bool {
        use Break::*;
        match (self.previous, next.gcb) {
            (Cr, Lf) => true,                                         // GB3
            (Control | Cr | Lf, _) | (_, Control | Cr | Lf) => false, // GB4, GB5
            (L, L | V | Lv | Lvt) => true,                            // GB6
            (Lv | V, V | T) => true,                                  // GB7
            (Lvt | T, T) => true,                                     // GB8
            (_, Extend | Zwj | SpacingMark) | (Prepend, _) => true,   // GB9, GB9a, GB9b
            _ if next.conjunct == Conjunct::Consonant && self.after_consonant && self.linked => {
                true
            } // GB9c
            (Zwj, _) if next.pictographic && self.zwj_after_pictographic => true, // GB11
            (RegionalIndicator, RegionalIndicator) => self.regional_run % 2 == 1, // GB12, GB13
            _ => false,                                               // GB999
        }
    }

    /// Advance past `c`.
    fn take(&mut self, c: Props) {
        self.zwj_after_pictographic = c.gcb == Break::Zwj && self.pictographic;
        self.pictographic = c.pictographic || (self.pictographic && c.gcb == Break::Extend);
        self.regional_run = if c.gcb == Break::RegionalIndicator {
            self.regional_run + 1
        } else {
            0
        };
        match c.conjunct {
            Conjunct::Consonant => {
                self.after_consonant = true;
                self.linked = false;
            }
            Conjunct::Linker if self.after_consonant => self.linked = true,
            Conjunct::Extend if self.after_consonant => {}
            _ => {
                self.after_consonant = false;
                self.linked = false;
            }
        }
        self.previous = c.gcb;
    }
}

/// The byte length of the grapheme cluster at the start of `text` (non-empty).
#[inline]
fn cluster_len(text: &str) -> usize {
    let bytes = text.as_bytes();
    // ASCII fast path: a printable ASCII character followed by ASCII (or the
    // end) is a cluster by itself, unless CR LF.
    if bytes[0] >= 0x20 && bytes[0] < 0x7f && bytes.get(1).is_none_or(|&b| b < 0x80) {
        return 1;
    }
    cluster_len_by_rules(text)
}

/// [`cluster_len`] past the ASCII fast path: the UAX #29 rules (out of line,
/// so callers inline only the fast path).
#[inline(never)]
fn cluster_len_by_rules(text: &str) -> usize {
    let mut chars = text.char_indices();
    let (_, first) = chars.next().expect("non-empty");
    let mut state = State::new(props(first));
    for (at, c) in chars {
        let next = props(c);
        if !state.joins(next) {
            return at;
        }
        state.take(next);
    }
    text.len()
}

/// The extended grapheme clusters of `text`.
pub fn graphemes(text: &str) -> Graphemes<'_> {
    Graphemes { rest: text }
}

/// The extended grapheme clusters of `text` with their byte offsets.
pub fn grapheme_indices(text: &str) -> GraphemeIndices<'_> {
    GraphemeIndices { text, at: 0 }
}

/// See [`graphemes`].
#[derive(Clone)]
pub struct Graphemes<'a> {
    rest: &'a str,
}

impl<'a> Iterator for Graphemes<'a> {
    type Item = &'a str;
    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() {
            return None;
        }
        let (cluster, rest) = self.rest.split_at(cluster_len(self.rest));
        self.rest = rest;
        Some(cluster)
    }
}

/// See [`grapheme_indices`].
#[derive(Clone)]
pub struct GraphemeIndices<'a> {
    text: &'a str,
    at: usize,
}

impl<'a> Iterator for GraphemeIndices<'a> {
    type Item = (usize, &'a str);
    #[inline]
    fn next(&mut self) -> Option<(usize, &'a str)> {
        let rest = &self.text[self.at..];
        if rest.is_empty() {
            return None;
        }
        let start = self.at;
        self.at += cluster_len(rest);
        Some((start, &self.text[start..self.at]))
    }
}
