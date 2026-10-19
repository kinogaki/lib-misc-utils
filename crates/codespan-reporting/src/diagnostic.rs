//! Diagnostics: a severity, a message, labelled spans of source and notes.

use core::ops::Range;

/// How serious a diagnostic is.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Help,
    Note,
    Warning,
    Error,
    Bug,
}

impl Severity {
    fn word(self) -> &'static str {
        match self {
            Severity::Help => "help",
            Severity::Note => "note",
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Bug => "bug",
        }
    }
}

impl core::fmt::Display for Severity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.word())
    }
}

/// A primary label marks the cause (`^`); a secondary one gives context (`-`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

/// A labelled byte range of one file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label<FileId> {
    pub style: LabelStyle,
    pub file_id: FileId,
    pub range: Range<usize>,
    pub message: String,
}

impl<FileId> Label<FileId> {
    pub fn new(
        style: LabelStyle,
        file_id: FileId,
        range: impl Into<Range<usize>>,
    ) -> Label<FileId> {
        Label {
            style,
            file_id,
            range: range.into(),
            message: String::new(),
        }
    }

    pub fn primary(file_id: FileId, range: impl Into<Range<usize>>) -> Label<FileId> {
        Label::new(LabelStyle::Primary, file_id, range)
    }

    pub fn secondary(file_id: FileId, range: impl Into<Range<usize>>) -> Label<FileId> {
        Label::new(LabelStyle::Secondary, file_id, range)
    }

    pub fn with_message(mut self, message: impl ToString) -> Label<FileId> {
        self.message = message.to_string();
        self
    }
}

/// A message about source code, with labels and notes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic<FileId> {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub labels: Vec<Label<FileId>>,
    pub notes: Vec<String>,
}

impl<FileId> Diagnostic<FileId> {
    pub fn new(severity: Severity) -> Diagnostic<FileId> {
        Diagnostic {
            severity,
            code: None,
            message: String::new(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn bug() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Bug)
    }

    pub fn error() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Error)
    }

    pub fn warning() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Warning)
    }

    pub fn note() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Note)
    }

    pub fn help() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Help)
    }

    pub fn with_code(mut self, code: impl ToString) -> Diagnostic<FileId> {
        self.code = Some(code.to_string());
        self
    }

    pub fn with_message(mut self, message: impl ToString) -> Diagnostic<FileId> {
        self.message = message.to_string();
        self
    }

    pub fn with_label(mut self, label: Label<FileId>) -> Diagnostic<FileId> {
        self.labels.push(label);
        self
    }

    pub fn with_labels(mut self, mut labels: Vec<Label<FileId>>) -> Diagnostic<FileId> {
        self.labels.append(&mut labels);
        self
    }

    pub fn with_notes(mut self, mut notes: Vec<String>) -> Diagnostic<FileId> {
        self.notes.append(&mut notes);
        self
    }
}
