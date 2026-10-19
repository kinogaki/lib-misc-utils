# codespan-reporting (Kinogaki)

Ours, not upstream codespan-reporting. It is published under the crates.io
name (`0.13.1+kinogaki`) for the frozen naga.

It renders naga's shader diagnostics as plain text in codespan's rich layout:
one message, labelled source spans and notes. It has the `Diagnostic`,
`Label`, `Files` and `SimpleFile` types and `term::emit_to_string` /
`emit_to_io_write`. There are no colors or terminal handling. Columns count
characters with tabs expanded, so no Unicode width tables are needed (this
is how unicode-width left the tree).

For naga's single-line labels the text is byte-identical to upstream's. That
was checked on parse, type and literal-range errors. tests/render.rs pins the
layout.
