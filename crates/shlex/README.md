# shlex (Kinogaki)

Ours, not upstream shlex. It is published under the crates.io name
(`2.0.1+kinogaki`) for the frozen cc, blake3's build script, which splits
CFLAGS-style variables.

`Shlex` splits words as a POSIX shell would, without expansions.
tests/split.rs covers quotes, escapes, comments and errors.
