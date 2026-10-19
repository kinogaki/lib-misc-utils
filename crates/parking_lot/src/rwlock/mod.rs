//! A writer-preferring reader-writer lock.
//!
//! The lock's state is one atomic word: a write bit, a writer-waiting bit, a
//! parked bit and the reader count. Taking an uncontended lock is one
//! compare-exchange; releasing it is one atomic operation, plus a wake-up when
//! the parked bit says someone sleeps. Waiters sleep on a std mutex and
//! condition variable (`Parking`):
//!
//! - A waiter holds the parking mutex while it sets the parked bit, re-checks
//!   the state and starts to wait, and a releaser takes the parking mutex
//!   before it notifies. A release that happens before the parked bit is set
//!   is seen by the re-check; one after it sees the bit and notifies once the
//!   waiter sleeps. No wake-up is lost.
//! - Writers that must wait announce themselves with the writer-waiting bit;
//!   readers do not enter while it is set, so a stream of readers cannot
//!   starve a writer.
//!
//! The data lives in a [`cell::DataCell`], the crate's only unsafe code: the
//! guards above are what make its shared and exclusive borrows sound.

mod cell;

use cell::DataCell;
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};

const WRITER: usize = 1;
const WRITER_WAITING: usize = 2;
const PARKED: usize = 4;
const READER: usize = 8;
const READERS: usize = !(READER - 1);

/// A reader may enter while no writer holds the lock or waits for it.
fn reader_may_enter(state: usize) -> bool {
    state & (WRITER | WRITER_WAITING) == 0
}

/// A writer may enter while nobody holds the lock.
fn writer_may_enter(state: usize) -> bool {
    state & (WRITER | READERS) == 0
}

/// Sleeping waiters: how many sleep, and how many of them are writers.
#[derive(Default)]
struct Parking {
    counts: std::sync::Mutex<Counts>,
    wake: std::sync::Condvar,
}

#[derive(Default)]
struct Counts {
    sleeping: usize,
    writers_waiting: usize,
}

/// A reader-writer lock: any number of readers, or one writer.
pub struct RwLock<T: ?Sized> {
    state: AtomicUsize,
    parking: Parking,
    data: DataCell<T>,
}

/// `RwLock::new` in const contexts.
pub const fn const_rwlock<T>(value: T) -> RwLock<T> {
    RwLock::new(value)
}

impl<T> RwLock<T> {
    pub const fn new(value: T) -> RwLock<T> {
        RwLock {
            state: AtomicUsize::new(0),
            parking: Parking {
                counts: std::sync::Mutex::new(Counts {
                    sleeping: 0,
                    writers_waiting: 0,
                }),
                wake: std::sync::Condvar::new(),
            },
            data: DataCell::new(value),
        }
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: Default> Default for RwLock<T> {
    fn default() -> RwLock<T> {
        RwLock::new(T::default())
    }
}

impl<T> From<T> for RwLock<T> {
    fn from(value: T) -> RwLock<T> {
        RwLock::new(value)
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Shared access; waits while a writer holds the lock or waits for it.
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        if !self.try_enter_read() {
            self.read_slow();
        }
        RwLockReadGuard { lock: self }
    }

    /// Shared access if no writer holds the lock or waits for it.
    #[inline]
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        self.try_enter_read()
            .then(|| RwLockReadGuard { lock: self })
    }

    /// Exclusive access; waits until no reader or writer holds the lock.
    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        if !self.try_enter_write() {
            self.write_slow();
        }
        RwLockWriteGuard { lock: self }
    }

    /// Exclusive access if nobody holds the lock.
    #[inline]
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        self.try_enter_write()
            .then(|| RwLockWriteGuard { lock: self })
    }

    /// Whether a reader or writer holds the lock (a snapshot; for assertions).
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.state.load(Ordering::Relaxed) & (WRITER | READERS) != 0
    }

    /// Whether a writer holds the lock (a snapshot; for assertions).
    #[inline]
    pub fn is_locked_exclusive(&self) -> bool {
        self.state.load(Ordering::Relaxed) & WRITER != 0
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    #[inline]
    fn try_enter_read(&self) -> bool {
        let mut state = self.state.load(Ordering::Relaxed);
        loop {
            if !reader_may_enter(state) {
                return false;
            }
            let next = state
                .checked_add(READER)
                .expect("parking_lot: too many readers");
            match self.state.compare_exchange_weak(
                state,
                next,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(now) => state = now,
            }
        }
    }

    #[inline]
    fn try_enter_write(&self) -> bool {
        let mut state = self.state.load(Ordering::Relaxed);
        loop {
            if !writer_may_enter(state) {
                return false;
            }
            match self.state.compare_exchange_weak(
                state,
                state | WRITER,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(now) => state = now,
            }
        }
    }

    #[cold]
    fn read_slow(&self) {
        let mut counts = self.parking.lock();
        loop {
            if self.try_enter_read() {
                return;
            }
            counts = self.sleep(counts, reader_may_enter);
        }
    }

    #[cold]
    fn write_slow(&self) {
        let mut counts = self.parking.lock();
        counts.writers_waiting += 1;
        self.state.fetch_or(WRITER_WAITING, Ordering::Relaxed);
        loop {
            let state = self.state.load(Ordering::Relaxed);
            if writer_may_enter(state) {
                // The last waiting writer clears the bit as it enters.
                let mut next = state | WRITER;
                if counts.writers_waiting == 1 {
                    next &= !WRITER_WAITING;
                }
                if self
                    .state
                    .compare_exchange(state, next, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok()
                {
                    counts.writers_waiting -= 1;
                    return;
                }
                continue;
            }
            counts = self.sleep(counts, writer_may_enter);
        }
    }

    /// Sleep until a release wakes us. The parked bit is set first and the
    /// state checked again with `may_enter` (a reader's or a writer's
    /// condition): a release between the caller's failed attempt and the
    /// parked bit saw no parked bit and will not wake us, so we return at once
    /// and the caller tries again.
    fn sleep<'a>(
        &'a self,
        mut counts: std::sync::MutexGuard<'a, Counts>,
        may_enter: fn(usize) -> bool,
    ) -> std::sync::MutexGuard<'a, Counts> {
        let state = self.state.fetch_or(PARKED, Ordering::Relaxed);
        if may_enter(state) {
            if counts.sleeping == 0 {
                self.state.fetch_and(!PARKED, Ordering::Relaxed);
            }
            return counts;
        }
        counts.sleeping += 1;
        counts = self
            .parking
            .wake
            .wait(counts)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        counts.sleeping -= 1;
        if counts.sleeping == 0 {
            self.state.fetch_and(!PARKED, Ordering::Relaxed);
        }
        counts
    }

    /// Release one read.
    #[inline]
    fn unlock_read(&self) {
        let before = self.state.fetch_sub(READER, Ordering::Release);
        debug_assert!(
            before & READERS != 0,
            "parking_lot: read unlock without a reader"
        );
        // Writers wait for the last reader; readers never wait for a reader.
        if before & PARKED != 0 && before & READERS == READER {
            self.parking.wake_all();
        }
    }

    /// Release the write.
    #[inline]
    fn unlock_write(&self) {
        let before = self.state.fetch_and(!WRITER, Ordering::Release);
        if before & PARKED != 0 {
            self.parking.wake_all();
        }
    }
}

impl Parking {
    fn lock(&self) -> std::sync::MutexGuard<'_, Counts> {
        self.counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Wake every sleeper; taking the mutex first orders the wake-up after
    /// any waiter's parked-bit check.
    fn wake_all(&self) {
        let _counts = self.lock();
        self.wake.notify_all();
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_read() {
            Some(guard) => f.debug_struct("RwLock").field("data", &&*guard).finish(),
            None => f
                .debug_struct("RwLock")
                .field("data", &format_args!("<locked>"))
                .finish(),
        }
    }
}

/// Shared access to an [`RwLock`]'s data until dropped (or forgotten and
/// released with `RwLock::force_unlock_read`).
#[must_use = "the lock is released when the guard is dropped"]
pub struct RwLockReadGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

/// Exclusive access to an [`RwLock`]'s data until dropped.
#[must_use = "the lock is released when the guard is dropped"]
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.unlock_read();
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.unlock_write();
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        self.lock.data.shared(self)
    }
}

impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        self.lock.data.exclusive_ref(self)
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        let lock = self.lock;
        lock.data.exclusive(self)
    }
}
