//! Kinogaki's own `shlex`, published under the crates.io name so the frozen
//! cc (blake3's build script, splitting CFLAGS-style variables) resolves to it
//! through `[patch.crates-io]`. It is not upstream shlex: it has the one type
//! cc uses.
//!
//! [`Shlex`] splits a string into words as a POSIX shell would, without
//! expansions: unquoted whitespace separates words; `'...'` is literal;
//! `"..."` is literal except `\` before `$`, `` ` ``, `"`, `\` or a newline;
//! an unquoted `\` takes the next character literally (and joins lines
//! before a newline); `#` at the start of a word comments out the rest of its
//! line. An unterminated quote or a trailing `\` ends the words and sets
//! `had_error`.

/// An iterator over the shell words of a string.
pub struct Shlex<'a> {
    chars: core::iter::Peekable<core::str::Chars<'a>>,
    /// Set when the input ended inside a quote or after a lone `\`.
    pub had_error: bool,
}

impl<'a> Shlex<'a> {
    pub fn new(input: &'a str) -> Shlex<'a> {
        Shlex {
            chars: input.chars().peekable(),
            had_error: false,
        }
    }

    /// The next word, or `None` at the end (or after an error).
    fn word(&mut self) -> Option<String> {
        // Skip separators and comments.
        loop {
            match self.chars.peek()? {
                ' ' | '\t' | '\n' => {
                    self.chars.next();
                }
                '#' => while self.chars.next_if(|&c| c != '\n').is_some() {},
                _ => break,
            }
        }
        let mut word = String::new();
        while let Some(c) = self.chars.next() {
            match c {
                ' ' | '\t' | '\n' => break,
                '\'' => loop {
                    match self.chars.next() {
                        Some('\'') => break,
                        Some(c) => word.push(c),
                        None => return self.error(),
                    }
                },
                '"' => loop {
                    match self.chars.next() {
                        Some('"') => break,
                        Some('\\') => match self.chars.next() {
                            Some(c @ ('$' | '`' | '"' | '\\')) => word.push(c),
                            Some('\n') => {}
                            Some(c) => {
                                word.push('\\');
                                word.push(c);
                            }
                            None => return self.error(),
                        },
                        Some(c) => word.push(c),
                        None => return self.error(),
                    }
                },
                '\\' => match self.chars.next() {
                    Some('\n') => {}
                    Some(c) => word.push(c),
                    None => return self.error(),
                },
                c => word.push(c),
            }
        }
        Some(word)
    }

    fn error(&mut self) -> Option<String> {
        self.had_error = true;
        // Nothing more is read after an error.
        while self.chars.next().is_some() {}
        None
    }
}

impl Iterator for Shlex<'_> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.had_error { None } else { self.word() }
    }
}

/// The words of `input`, or `None` when it has an unterminated quote or escape.
pub fn split(input: &str) -> Option<Vec<String>> {
    let mut words = Shlex::new(input);
    let out: Vec<String> = words.by_ref().collect();
    (!words.had_error).then_some(out)
}
