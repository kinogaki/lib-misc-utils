# cfg-if (Kinogaki)

Ours, not upstream cfg-if. It is published under the crates.io name
(`1.0.5+kinogaki`) for the frozen naga, wgpu, wgpu-hal, blake3 and
libloading.

`cfg_if!` keeps the first `if #[cfg(..)]` branch whose predicate holds. The
branches may hold items, statements or a tail expression (tests/branches.rs).
