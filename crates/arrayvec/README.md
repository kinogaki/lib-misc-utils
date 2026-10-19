# arrayvec (Kinogaki)

Ours, not upstream arrayvec. It is published under the crates.io name
(`0.7.8+kinogaki`) for the frozen naga, wgpu, wgpu-core, wgpu-hal and blake3.

It has `ArrayVec<T, CAP>`, `ArrayString<CAP>`, `CapacityError`, `Drain` and
`IntoIter`, with the methods and traits those crates call.

The inline storage, src/buffer.rs, is the crate's only unsafe code: a
`[MaybeUninit<T>; CAP]` whose first `len` slots are initialised, with push,
pop, truncate, slice views and drop. Everything else (insert, remove, drain,
retain, extend and conversions) is safe code over those. `ArrayString` is
safe throughout.

tests/array_vec.rs counts every element's drops through each operation.
