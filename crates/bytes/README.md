# kinogaki-bytes

Byte search in safe code: `find`, `rfind` and `count` for one byte value.
lib-obj's reader uses it for line feeds.

It reads eight bytes at a time as a `u64` with an exact zero-byte test
(no carries cross bytes), and tests four words per step on long slices.
tests/search.rs checks every length, position and alignment, and needles
that stress the trick.
