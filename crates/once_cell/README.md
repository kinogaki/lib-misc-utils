# once_cell (Kinogaki)

Ours, not upstream once_cell. It is published under the crates.io name
(`1.21.4+kinogaki`) for the frozen naga, wgpu-core and read-fonts.

It has `sync::OnceCell` (with the fallible `get_or_try_init` wgpu-core
uses), `sync::Lazy` and `race::OnceBox`, over std's `OnceLock` and
`LazyLock`.
