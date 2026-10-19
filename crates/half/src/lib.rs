//! Kinogaki's own `half`, published under the crates.io name so the frozen
//! naga resolves to it through `[patch.crates-io]`. It is not upstream half:
//! it has the IEEE 754 binary16 type [`f16`] with what naga calls (its
//! literals and constant folding), and with the `num-traits` feature our
//! num-traits implementations for it.
//!
//! Conversions are bit-exact IEEE 754 (round to nearest, ties to even; NaN
//! stays NaN with its payload's top bits and the quiet bit set), checked over
//! all 65,536 values and every rounding boundary (tests/conversions.rs).
//! Arithmetic computes in f32 and rounds once to f16; for +, -, ×, ÷ and %
//! that is the correctly rounded binary16 result (f32 has more than twice
//! binary16's precision plus two bits, so the double rounding is harmless).

mod convert;
mod f16_type;
#[cfg(feature = "num-traits")]
mod num_traits_impls;

pub use f16_type::f16;
