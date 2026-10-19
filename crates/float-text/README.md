# kinogaki-float-text

The shortest decimal text that reads back as the same `f64`, laid out as
zmij lays it out (plain decimal for exponents -5..=15, `d.ddde±N` outside).
The digits come from Schubfach's search. tests/against_std.rs compares them
with std's shortest digits on millions of values, and tests/tables.rs checks
the power-of-ten table with exact big integers.

Not yet used: it matches zmij byte for byte but is about 2x slower per value
in safe scalar code, so lib-obj keeps the frozen zmij until it is as fast.
