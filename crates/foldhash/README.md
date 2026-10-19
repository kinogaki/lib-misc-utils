# foldhash (Kinogaki)

Ours, not upstream foldhash. It is published under the crates.io name
(`0.2.0+kinogaki`) for the frozen hashbrown's default hasher
(`fast::RandomState`).

It is our own seeded folded-multiply hash. tests/quality.rs checks avalanche
on every output bit and even bucket and control-byte spread. Seeds are random
per process and per builder, as upstream's are.
