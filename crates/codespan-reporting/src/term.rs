//! Rendering diagnostics as text.

use crate::diagnostic::{Diagnostic, Label, LabelStyle};
use crate::files::{Error, Files};
use core::fmt::Write as _;

/// How much a diagnostic shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayStyle {
    /// The message, the labelled source lines and the notes.
    Rich,
    /// One `file:line:column: severity: message` line per primary label, then the notes.
    Medium,
    /// One `file:line:column: severity: message` line per primary label.
    Short,
}

/// Rendering options.
#[derive(Clone, Debug)]
pub struct Config {
    pub display_style: DisplayStyle,
    /// Columns a tab advances in source lines.
    pub tab_width: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            display_style: DisplayStyle::Rich,
            tab_width: 4,
        }
    }
}

/// Append `diagnostic` to `writer`.
pub fn emit_to_string<'files, F: Files<'files> + ?Sized>(
    writer: &mut String,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), Error> {
    match config.display_style {
        DisplayStyle::Rich => rich(writer, config, files, diagnostic),
        DisplayStyle::Medium => short(writer, files, diagnostic, true),
        DisplayStyle::Short => short(writer, files, diagnostic, false),
    }
}

/// `diagnostic` as a new string.
pub fn emit_into_string<'files, F: Files<'files> + ?Sized>(
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<String, Error> {
    let mut text = String::new();
    emit_to_string(&mut text, config, files, diagnostic)?;
    Ok(text)
}

/// Write `diagnostic` to an `io::Write`.
#[cfg(feature = "std")]
pub fn emit_to_io_write<'files, F: Files<'files> + ?Sized, W: std::io::Write + ?Sized>(
    writer: &mut W,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), Error> {
    let text = emit_into_string(config, files, diagnostic)?;
    writer.write_all(text.as_bytes())?;
    Ok(())
}

fn header<FileId>(out: &mut String, diagnostic: &Diagnostic<FileId>) -> Result<(), Error> {
    write!(out, "{}", diagnostic.severity)?;
    if let Some(code) = &diagnostic.code {
        write!(out, "[{code}]")?;
    }
    if !diagnostic.message.is_empty() {
        write!(out, ": {}", diagnostic.message)?;
    }
    out.push('\n');
    Ok(())
}

/// A label resolved against its file: zero-based lines, byte columns in them.
struct Placed<'d> {
    label_index: usize,
    style: LabelStyle,
    message: &'d str,
    start: (usize, usize),
    end: (usize, usize),
}

/// Clamp `index` into `source` and back to a code point boundary.
fn boundary(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while !source.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn place<'files, 'd, F: Files<'files> + ?Sized>(
    files: &'files F,
    source: &str,
    index: usize,
    label: &'d Label<F::FileId>,
) -> Result<Placed<'d>, Error> {
    let start_byte = boundary(source, label.range.start);
    let end_byte = boundary(source, label.range.end.max(label.range.start));
    let start_line = files.line_index(label.file_id, start_byte)?;
    let end_line = files.line_index(label.file_id, end_byte)?;
    let start_line_byte = files.line_range(label.file_id, start_line)?.start;
    let end_line_byte = files.line_range(label.file_id, end_line)?.start;
    Ok(Placed {
        label_index: index,
        style: label.style,
        message: &label.message,
        start: (start_line, start_byte - start_line_byte),
        end: (end_line, end_byte - end_line_byte),
    })
}

/// The line's text without its ending, tabs expanded.
fn line_text(source: &str, range: core::ops::Range<usize>, tab_width: usize) -> String {
    let text = source[range].trim_end_matches(['\n', '\r']);
    let mut out = String::new();
    for c in text.chars() {
        if c == '\t' {
            let pad = tab_width - out.chars().count() % tab_width.max(1);
            out.extend(core::iter::repeat_n(' ', pad));
        } else {
            out.push(c);
        }
    }
    out
}

/// The display column of byte `column` in a line (tabs expanded).
fn display_column(line: &str, column: usize, tab_width: usize) -> usize {
    let mut width = 0;
    for c in line[..boundary(line, column)].chars() {
        width = if c == '\t' {
            width + tab_width - width % tab_width.max(1)
        } else {
            width + 1
        };
    }
    width
}

fn rich<'files, F: Files<'files> + ?Sized>(
    out: &mut String,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), Error> {
    header(out, diagnostic)?;
    // Labels grouped by file, files in order of first mention.
    let mut file_ids: Vec<F::FileId> = Vec::new();
    for label in &diagnostic.labels {
        if !file_ids.contains(&label.file_id) {
            file_ids.push(label.file_id);
        }
    }
    let mut placed_files = Vec::new();
    let mut gutter = 0;
    for &file_id in &file_ids {
        let source = files.source(file_id)?;
        let source = source.as_ref();
        let mut placed = Vec::new();
        for (i, label) in diagnostic
            .labels
            .iter()
            .enumerate()
            .filter(|(_, l)| l.file_id == file_id)
        {
            let p = place(files, source, i, label)?;
            gutter = gutter.max((p.end.0 + 1).to_string().len());
            placed.push(p);
        }
        placed_files.push((file_id, placed));
    }
    let pad = " ".repeat(gutter);
    for (file_id, placed) in &placed_files {
        let source = files.source(*file_id)?;
        let source = source.as_ref();
        let lead = placed
            .iter()
            .find(|p| p.style == LabelStyle::Primary)
            .unwrap_or(&placed[0]);
        let lead_line = files.line_range(*file_id, lead.start.0)?;
        let lead_column = source[lead_line.start..lead_line.start + lead.start.1]
            .chars()
            .count()
            + 1;
        writeln!(
            out,
            "{pad} ┌─ {}:{}:{lead_column}",
            files.name(*file_id)?,
            lead.start.0 + 1
        )?;
        writeln!(out, "{pad} │")?;
        let mut lines: Vec<usize> = placed.iter().flat_map(|p| [p.start.0, p.end.0]).collect();
        lines.sort_unstable();
        lines.dedup();
        for (n, &line) in lines.iter().enumerate() {
            if n > 0 && line > lines[n - 1] + 1 {
                writeln!(out, "{pad} ·")?;
            }
            let range = files.line_range(*file_id, line)?;
            let raw = &source[range.clone()];
            let text = line_text(source, range, config.tab_width);
            writeln!(out, "{:>gutter$} │ {text}", line + 1)?;
            let width = text.chars().count();
            let mut marks: Vec<&Placed> = placed
                .iter()
                .filter(|p| p.start.0 == line || p.end.0 == line)
                .collect();
            marks.sort_by_key(|p| (p.start.1, p.label_index));
            for p in marks {
                let single = p.start.0 == p.end.0;
                let from = if p.start.0 == line {
                    display_column(raw, p.start.1, config.tab_width)
                } else {
                    0
                };
                let to = if p.end.0 == line {
                    display_column(raw, p.end.1, config.tab_width)
                } else {
                    width
                };
                let mark = if p.style == LabelStyle::Primary {
                    '^'
                } else {
                    '-'
                };
                let mut row = format!("{pad} │ {}", " ".repeat(from));
                row.extend(core::iter::repeat_n(mark, to.saturating_sub(from).max(1)));
                // A multi-line label's message goes under its last line.
                if !p.message.is_empty() && (single || p.end.0 == line) {
                    row.push(' ');
                    row.push_str(p.message);
                } else if !single && p.start.0 == line {
                    let _ = write!(row, " (to line {})", p.end.0 + 1);
                }
                writeln!(out, "{row}")?;
            }
        }
    }
    if !placed_files.is_empty() && !diagnostic.notes.is_empty() {
        writeln!(out, "{pad} │")?;
    }
    for note in &diagnostic.notes {
        let mut lines = note.lines();
        writeln!(out, "{pad} = {}", lines.next().unwrap_or(""))?;
        for more in lines {
            writeln!(out, "{pad}   {more}")?;
        }
    }
    out.push('\n');
    Ok(())
}

fn short<'files, F: Files<'files> + ?Sized>(
    out: &mut String,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
    notes: bool,
) -> Result<(), Error> {
    let mut any = false;
    for label in diagnostic
        .labels
        .iter()
        .filter(|l| l.style == LabelStyle::Primary)
    {
        let source = files.source(label.file_id)?;
        let start = boundary(source.as_ref(), label.range.start);
        let line = files.line_index(label.file_id, start)?;
        let line_start = files.line_range(label.file_id, line)?.start;
        let column = source.as_ref()[line_start..start].chars().count() + 1;
        write!(
            out,
            "{}:{}:{column}: ",
            files.name(label.file_id)?,
            line + 1
        )?;
        header(out, diagnostic)?;
        any = true;
    }
    if !any {
        header(out, diagnostic)?;
    }
    if notes {
        for note in &diagnostic.notes {
            writeln!(out, " = {note}")?;
        }
    }
    Ok(())
}
