# half (Kinogaki)

Ours, not upstream half. It is published under the crates.io name `half` with
version `2.7.1+kinogaki`, so the frozen naga resolves to it through
`[patch.crates-io]`.

It provides the IEEE 754 binary16 type `f16`: its constants, conversions,
comparisons, arithmetic, `Sum`/`Product` and formatting. With the
`num-traits` feature it also implements our num-traits for `f16`. That is all
naga uses (f16 literals and constant folding). Upstream's bf16, slices, vecs,
serde and bytemuck support are not here.

Conversions are bit-exact IEEE 754 (round to nearest, ties to even).
Arithmetic computes in f32 and rounds once, which is correctly rounded for
binary16.

Tests (tests/conversions.rs):

- every one of the 65,536 values widens exactly and round-trips;
- every rounding boundary between neighbouring values, from f32 and f64;
- 2M random inputs against a nearest-value reference;
- 1M random operand pairs against arithmetic in f64.

Before upstream was removed, ours matched it, and so the M-series hardware fp16
instructions, on all 2^32 f32 inputs.
