# num-traits (Kinogaki)

Ours, not upstream num-traits. It is published under the crates.io name
`num-traits` (`0.2.19+kinogaki`) for the frozen naga and our `half`.

It holds the traits those two use: `Zero`, `One`, `Num`, `Bounded`, `Signed`,
`ToPrimitive`, `FromPrimitive`, `NumCast`, `AsPrimitive`, `FloatCore`,
`Float`, `Real`, `FloatConst`, `PrimInt`, and the checked and wrapping
operations. They are implemented for the primitive types, with std's float
functions for `f32` and `f64`.

Conversions follow num-traits' rules, which naga's constant evaluator relies
on: float to integer truncates and is `None` when the result does not fit.
tests/cast.rs checks these rules at every boundary.
