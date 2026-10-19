//! The rendered text of naga-shaped diagnostics.

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{Files, SimpleFile};
use codespan_reporting::term::{self, Config, DisplayStyle};

const SOURCE: &str = "fn main() {\n    let x = ;\n\tlet y = 1;\n}\n";

fn render(diagnostic: &Diagnostic<()>, config: &Config) -> String {
    let file = SimpleFile::new("shader.wgsl", SOURCE);
    let mut text = String::new();
    term::emit_to_string(&mut text, config, &file, diagnostic).unwrap();
    text
}

#[test]
fn one_label_and_notes() {
    let at = SOURCE.find(';').unwrap();
    let diagnostic = Diagnostic::error()
        .with_message("expected expression, found ';'")
        .with_labels(vec![
            Label::primary((), at..at + 1).with_message("expected expression"),
        ])
        .with_notes(vec![
            "while parsing a declaration".into(),
            "two\nlines".into(),
        ]);
    assert_eq!(
        render(&diagnostic, &Config::default()),
        "error: expected expression, found ';'\n\
         \x20 ┌─ shader.wgsl:2:13\n\
         \x20 │\n\
         2 │     let x = ;\n\
         \x20 │             ^ expected expression\n\
         \x20 │\n\
         \x20 = while parsing a declaration\n\
         \x20 = two\n\
         \x20   lines\n\n"
    );
}

#[test]
fn tabs_expand_and_gaps_show() {
    let y = SOURCE.find('y').unwrap();
    let main = SOURCE.find("main").unwrap();
    let diagnostic = Diagnostic::warning()
        .with_message("unused")
        .with_labels(vec![
            Label::primary((), y..y + 1).with_message("never read"),
            Label::secondary((), main..main + 4).with_message("in this function"),
        ]);
    assert_eq!(
        render(&diagnostic, &Config::default()),
        "warning: unused\n\
         \x20 ┌─ shader.wgsl:3:6\n\
         \x20 │\n\
         1 │ fn main() {\n\
         \x20 │    ---- in this function\n\
         \x20 ·\n\
         3 │     let y = 1;\n\
         \x20 │         ^ never read\n\n"
    );
}

#[test]
fn multi_line_label_and_empty_range() {
    let open = SOURCE.find('{').unwrap();
    let close = SOURCE.rfind('}').unwrap();
    let diagnostic = Diagnostic::error().with_message("body").with_labels(vec![
        Label::primary((), open..close + 1).with_message("this block"),
        Label::secondary((), 0..0),
    ]);
    assert_eq!(
        render(&diagnostic, &Config::default()),
        "error: body\n\
         \x20 ┌─ shader.wgsl:1:11\n\
         \x20 │\n\
         1 │ fn main() {\n\
         \x20 │ -\n\
         \x20 │           ^ (to line 4)\n\
         \x20 ·\n\
         4 │ }\n\
         \x20 │ ^ this block\n\n"
    );
}

#[test]
fn message_only_and_short_styles() {
    let diagnostic = Diagnostic::<()>::error().with_message("bad module");
    assert_eq!(
        render(&diagnostic, &Config::default()),
        "error: bad module\n\n"
    );
    let at = SOURCE.find("let").unwrap();
    let diagnostic = Diagnostic::error()
        .with_message("m")
        .with_labels(vec![Label::primary((), at..at + 3)])
        .with_notes(vec!["n".into()]);
    let short = Config {
        display_style: DisplayStyle::Short,
        ..Config::default()
    };
    assert_eq!(render(&diagnostic, &short), "shader.wgsl:2:5: error: m\n");
    let medium = Config {
        display_style: DisplayStyle::Medium,
        ..Config::default()
    };
    assert_eq!(
        render(&diagnostic, &medium),
        "shader.wgsl:2:5: error: m\n = n\n"
    );
}

#[test]
fn line_lookup() {
    let file = SimpleFile::new("f", "ab\ncd\n");
    assert_eq!(file.line_index((), 0).unwrap(), 0);
    assert_eq!(file.line_index((), 2).unwrap(), 0);
    assert_eq!(file.line_index((), 3).unwrap(), 1);
    assert_eq!(file.line_index((), 6).unwrap(), 2);
    assert_eq!(file.line_range((), 1).unwrap(), 3..6);
    assert_eq!(file.line_range((), 2).unwrap(), 6..6);
    assert!(file.line_range((), 3).is_err());
}
