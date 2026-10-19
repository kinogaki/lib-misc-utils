# rustc-hash (Kinogaki)

Ours, not upstream rustc-hash. It is published under the crates.io name
(`1.1.0+kinogaki`) for the frozen naga, naga-types and wgpu-core.

`FxHasher` produces the 1.1 hashes bit for bit (tests/known_hashes.rs), so
map iteration orders stay the same. It also has the `FxHashMap`,
`FxHashSet` and `FxBuildHasher` aliases.
