# libm (Kinogaki)

Ours, not upstream libm. It is published under the crates.io name `libm`
(`0.2.16+kinogaki`) for the frozen naga.

It has the four functions naga calls directly: `asinh`, `acosh`, `rint` and
`rintf`. naga's other float maths goes through our num-traits, which uses
std.

`rint`/`rintf` round to nearest, ties to even (exact). `asinh` and `acosh`
use the classic range split over std's `ln`, `ln_1p` and `sqrt`.

tests/accuracy.rs compares them against references computed to 60 digits
(tools/reference.py): every case is within 1 ulp.
