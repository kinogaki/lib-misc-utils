# bit-vec (Kinogaki)

Ours, not upstream bit-vec. It is published under the crates.io name
(`0.9.1+kinogaki`) for the frozen wgpu-core and our bit-set.

It has `BitVec<B>` with the blocks, the bulk operations, and the `iter_mut` /
`get_mut` write-back proxies naga uses. As upstream, `clear` zeroes the bits
and keeps the length. Safe code.
