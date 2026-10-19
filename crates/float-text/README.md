# kinogaki-float-text

The shortest decimal text that reads back as the same `f64`, byte for byte
as zmij writes it: plain decimal for exponents -5..=15, `d.ddde±N` outside.
The digits come from one 128 × 64-bit scaling by a power of ten
(src/shortest.rs, the method zmij uses), and are written eight at a time.

Tests:

- tests/against_std.rs compares the digits with std's shortest on millions
  of values: all exponents, powers of two and their neighbours, and short
  decimals. They differ only at exact ties, where we round to even.
- tests/tables.rs checks the power-of-ten table with exact big integers.
- tests/digits.rs checks the digit routine for all of 0..10^8.
