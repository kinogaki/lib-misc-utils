//! Source files: names, text and line lookup.

use core::ops::Range;

/// A failed file lookup or write.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    FileMissing,
    IndexTooLarge {
        given: usize,
        max: usize,
    },
    LineTooLarge {
        given: usize,
        max: usize,
    },
    ColumnTooLarge {
        given: usize,
        max: usize,
    },
    InvalidCharBoundary {
        given: usize,
    },
    #[cfg(feature = "std")]
    Io(std::io::Error),
    Fmt,
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<core::fmt::Error> for Error {
    fn from(_: core::fmt::Error) -> Error {
        Error::Fmt
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::FileMissing => write!(f, "file missing"),
            Error::IndexTooLarge { given, max } => {
                write!(f, "invalid index {given}, maximum index is {max}")
            }
            Error::LineTooLarge { given, max } => {
                write!(f, "invalid line {given}, maximum line is {max}")
            }
            Error::ColumnTooLarge { given, max } => {
                write!(f, "invalid column {given}, maximum column {max}")
            }
            Error::InvalidCharBoundary { .. } => write!(f, "index is not a code point boundary"),
            #[cfg(feature = "std")]
            Error::Io(error) => write!(f, "{error}"),
            Error::Fmt => write!(f, "formatting error"),
        }
    }
}

impl core::error::Error for Error {}

/// A one-based line and column.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Location {
    pub line_number: usize,
    pub column_number: usize,
}

/// A database of source files.
pub trait Files<'a> {
    type FileId: 'a + Copy + PartialEq;
    type Name: 'a + core::fmt::Display;
    type Source: 'a + AsRef<str>;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error>;
    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error>;
    /// The zero-based line holding `byte_index`.
    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error>;
    /// The byte range of a zero-based line, with its line ending.
    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error>;
}

/// The byte offsets where the lines of `source` start.
pub fn line_starts(source: &str) -> impl '_ + Iterator<Item = usize> {
    core::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}

/// One named source file.
#[derive(Debug, Clone)]
pub struct SimpleFile<Name, Source> {
    name: Name,
    source: Source,
    line_starts: Vec<usize>,
}

impl<Name: core::fmt::Display, Source: AsRef<str>> SimpleFile<Name, Source> {
    pub fn new(name: Name, source: Source) -> SimpleFile<Name, Source> {
        let line_starts = line_starts(source.as_ref()).collect();
        SimpleFile {
            name,
            source,
            line_starts,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    fn line_start(&self, line_index: usize) -> Result<usize, Error> {
        use core::cmp::Ordering;
        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Ok(self.line_starts[line_index]),
            Ordering::Equal => Ok(self.source.as_ref().len()),
            Ordering::Greater => Err(Error::LineTooLarge {
                given: line_index,
                max: self.line_starts.len() - 1,
            }),
        }
    }
}

impl<'a, Name, Source> Files<'a> for SimpleFile<Name, Source>
where
    Name: 'a + core::fmt::Display + Clone,
    Source: 'a + AsRef<str>,
{
    type FileId = ();
    type Name = Name;
    type Source = &'a str;

    fn name(&'a self, (): ()) -> Result<Name, Error> {
        Ok(self.name.clone())
    }

    fn source(&'a self, (): ()) -> Result<&'a str, Error> {
        Ok(self.source.as_ref())
    }

    fn line_index(&'a self, (): (), byte_index: usize) -> Result<usize, Error> {
        Ok(self
            .line_starts
            .partition_point(|&start| start <= byte_index)
            .saturating_sub(1))
    }

    fn line_range(&'a self, (): (), line_index: usize) -> Result<Range<usize>, Error> {
        Ok(self.line_start(line_index)?..self.line_start(line_index + 1)?)
    }
}
