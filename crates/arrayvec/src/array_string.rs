//! The fixed-capacity string, in safe code: UTF-8 bytes in an inline array.

use crate::CapacityError;
use core::fmt;
use core::ops::Deref;

/// A string in at most `CAP` inline bytes.
#[derive(Clone, Copy)]
pub struct ArrayString<const CAP: usize> {
    bytes: [u8; CAP],
    len: usize,
}

impl<const CAP: usize> ArrayString<CAP> {
    pub const fn new() -> ArrayString<CAP> {
        ArrayString {
            bytes: [0; CAP],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub const fn is_full(&self) -> bool {
        self.len == CAP
    }

    pub const fn remaining_capacity(&self) -> usize {
        CAP - self.len
    }

    /// Append `c`. Panics when it does not fit.
    #[track_caller]
    pub fn push(&mut self, c: char) {
        self.try_push(c)
            .expect("ArrayString::push: capacity exceeded");
    }

    pub fn try_push(&mut self, c: char) -> Result<(), CapacityError<char>> {
        let mut utf8 = [0; 4];
        let encoded = c.encode_utf8(&mut utf8);
        self.append(encoded.as_bytes())
            .map_err(|()| CapacityError::new(c))
    }

    /// Append `s`. Panics when it does not fit.
    #[track_caller]
    pub fn push_str(&mut self, s: &str) {
        self.try_push_str(s)
            .expect("ArrayString::push_str: capacity exceeded");
    }

    pub fn try_push_str<'a>(&mut self, s: &'a str) -> Result<(), CapacityError<&'a str>> {
        self.append(s.as_bytes())
            .map_err(|()| CapacityError::new(s))
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_str(&self) -> &str {
        // Only whole UTF-8 sequences are ever appended.
        core::str::from_utf8(&self.bytes[..self.len]).expect("ArrayString holds UTF-8")
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), ()> {
        if bytes.len() > CAP - self.len {
            return Err(());
        }
        self.bytes[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

impl<const CAP: usize> Default for ArrayString<CAP> {
    fn default() -> ArrayString<CAP> {
        ArrayString::new()
    }
}

impl<const CAP: usize> Deref for ArrayString<CAP> {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAP: usize> AsRef<str> for ArrayString<CAP> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const CAP: usize> fmt::Debug for ArrayString<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const CAP: usize> fmt::Display for ArrayString<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<const CAP: usize> fmt::Write for ArrayString<CAP> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_push_str(s).map_err(|_| fmt::Error)
    }
}

impl<const CAP: usize> PartialEq for ArrayString<CAP> {
    fn eq(&self, other: &ArrayString<CAP>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const CAP: usize> PartialEq<str> for ArrayString<CAP> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const CAP: usize> PartialEq<&str> for ArrayString<CAP> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const CAP: usize> Eq for ArrayString<CAP> {}

impl<const CAP: usize> core::hash::Hash for ArrayString<CAP> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<const CAP: usize> PartialOrd for ArrayString<CAP> {
    fn partial_cmp(&self, other: &ArrayString<CAP>) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const CAP: usize> Ord for ArrayString<CAP> {
    fn cmp(&self, other: &ArrayString<CAP>) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}
