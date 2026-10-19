# parking_lot (Kinogaki)

Ours, not upstream parking_lot. It is published under the crates.io name
(`0.12.5+kinogaki`) for the frozen wgpu, wgpu-core and wgpu-hal.

- `Mutex` and `Condvar` wrap std's, with parking_lot's API (no poisoning;
  waits take the guard by `&mut`, with timeouts and predicates).
- `RwLock` is writer-preferring: once a writer waits, new readers wait
  behind it, so writers are never starved. Its state is one atomic word; an
  uncontended read or write is one compare-exchange, and waiters park on a
  std mutex and condition variable.
- `RwLockReadGuard` can be forgotten and its read released with
  `force_unlock_read`. wgpu-core's snatch lock does this, and std's RwLock
  cannot, which is why this lock is ours rather than std's.

The only unsafe code is the RwLock's data cell, src/rwlock/cell.rs, where
every block states its argument. lock_api, parking_lot_core and scopeguard are
not needed.

tests/locks.rs covers:

- exclusion under contention;
- readers overlapping;
- recursive reads with no writer waiting;
- a waiting writer going before new readers;
- writers not starved by streams of readers;
- forget plus `force_unlock_read`;
- no poisoning;
- Condvar timeouts and predicates;
- mixed try/blocking stress.
