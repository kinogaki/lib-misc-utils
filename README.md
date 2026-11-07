# lib-misc-utils

Kinogaki's own small crates: replacements for the helper crates in Kinogaki
Studio's dependency tree. The owner's direction (2026-10-18): own the code,
except high-threat things like cryptography.

Two kinds live here:

- **Our crates, for our code** (`kinogaki-*`): `kinogaki-unicode` (UAX #29
  graphemes and word bounds, combining marks, tables generated from the UCD),
  `kinogaki-bytes` (byte search) and `kinogaki-float-text` (shortest
  round-trip float text).
- **Our crates under crates.io names**, for the frozen crates that use them
  (wgpu, naga, harfrust, hashbrown, blake3 and cc): half, num-traits, libm,
  codespan-reporting, parking_lot, arrayvec, smallvec, once_cell, cfg-if,
  static_assertions, document-features, profiling, rustc-hash, foldhash,
  bit-vec, bit-set and shlex. Each implements only the API those crates call,
  and its version is the upstream one plus `+kinogaki`. Studio and the
  libraries point at them through the `[patch.crates-io]` block that lib-ui's
  `tools/vendor.py patch` writes, so the frozen consumers stay unedited.
  Each crate's README says what it covers and how it is tested.

When wgpu or naga are re-frozen, build them against these crates. A new call
into an API that is missing here fails to compile, and the fix is to add it
with a test.

Rules: unsafe is forbidden, except the documented cores of arrayvec
(src/buffer.rs) and parking_lot (src/rwlock/cell.rs). No dependencies outside
this repository. Studio checks the repository with `tools/check_all.py`, and
`tools/check_architecture.py` enforces the rules above.

## Licence

MIT OR Apache-2.0, at your option: see LICENSE-MIT and LICENSE-APACHE.
