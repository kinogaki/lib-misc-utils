//! Kinogaki's own `codespan-reporting`, published under the crates.io name so
//! the frozen naga resolves to it through `[patch.crates-io]`. It is not
//! upstream codespan-reporting: it renders naga's shader diagnostics (one
//! message, labelled source spans, notes) as plain text in codespan's rich
//! layout:
//!
//! ```text
//! error: expected expression, found ';'
//!   ┌─ shader.wgsl:3:13
//!   │
//! 3 │     let x = ;
//!   │             ^ expected expression
//!   │
//!   = while parsing a declaration
//! ```
//!
//! No colors and no terminal handling. Columns count characters, with tabs
//! expanded to `Config::tab_width` spaces (shader sources are ASCII, so no
//! Unicode width tables are needed).

pub mod diagnostic;
pub mod files;
pub mod term;
