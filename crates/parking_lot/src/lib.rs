//! Kinogaki's own `parking_lot`, published under the crates.io name so the
//! frozen wgpu, wgpu-core and wgpu-hal resolve to it through
//! `[patch.crates-io]`. It is not upstream parking_lot: it has the lock API
//! those crates call, with parking_lot's semantics where they rely on them:
//!
//! - no poisoning: a panic while a lock is held leaves it usable;
//! - `Condvar` waits take the guard by `&mut` and return a timeout result;
//! - `RwLock` is writer-preferring (task-fair): once a writer waits, new
//!   readers wait behind it, so writers are never starved and a thread that
//!   reads recursively while a writer waits deadlocks, as with parking_lot.
//!   (wgpu-core never re-reads a lock it holds; its lock ranks forbid it.)
//! - `RwLockReadGuard` can be forgotten and its read released later with
//!   `RwLock::force_unlock_read` (wgpu-core's snatch lock does this).
//!
//! `Mutex` and `Condvar` wrap std's. `RwLock` keeps its state in one atomic
//! word (an uncontended read or write is one compare-exchange) and parks
//! waiters on a std mutex and condition variable; its data cell is the crate's
//! only unsafe code (src/rwlock/cell.rs).

mod condvar;
mod mutex;
mod rwlock;

pub use condvar::{Condvar, WaitTimeoutResult};
pub use mutex::{Mutex, MutexGuard, const_mutex};
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard, const_rwlock};
