# smallvec (Kinogaki)

Ours, not upstream smallvec. It is published under the crates.io name
(`1.16.3+kinogaki`) for the frozen wgpu, wgpu-core, wgpu-hal and harfrust.
kinogaki-ui uses it too.

`SmallVec<[T; N]>` keeps up to N elements inline (in our arrayvec). When one
more does not fit, it moves them to a `Vec`. As upstream, it stays on the
heap until `shrink_to_fit`. It is safe code; the only unsafe code underneath
is arrayvec's documented buffer.

tests/small_vec.rs checks editing inline and spilled, drops and the
`smallvec!` macro.
