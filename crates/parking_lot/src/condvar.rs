//! A condition variable for [`Mutex`](crate::Mutex), on std's `Condvar`.

use crate::MutexGuard;
use std::time::{Duration, Instant};

/// A condition variable. Waits take the guard by `&mut` and hold the lock
/// again when they return.
#[derive(Default, Debug)]
pub struct Condvar {
    inner: std::sync::Condvar,
}

/// Whether a timed wait returned because its time ran out.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct WaitTimeoutResult(bool);

impl WaitTimeoutResult {
    #[inline]
    pub fn timed_out(self) -> bool {
        self.0
    }
}

impl Condvar {
    #[inline]
    pub const fn new() -> Condvar {
        Condvar {
            inner: std::sync::Condvar::new(),
        }
    }

    /// Wake one waiting thread. Returns `true` (std cannot tell whether a
    /// thread was waiting; no caller of ours uses the result).
    #[inline]
    pub fn notify_one(&self) -> bool {
        self.inner.notify_one();
        true
    }

    /// Wake every waiting thread. Returns 0 (std cannot count them; no caller
    /// of ours uses the result).
    #[inline]
    pub fn notify_all(&self) -> usize {
        self.inner.notify_all();
        0
    }

    /// Release the lock, sleep until notified (or spuriously), lock again.
    pub fn wait<T>(&self, guard: &mut MutexGuard<'_, T>) {
        let held = take(guard);
        let held = self
            .inner
            .wait(held)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.inner = Some(held);
    }

    /// [`Condvar::wait`] until `condition` is false.
    pub fn wait_while<T, F: FnMut(&mut T) -> bool>(
        &self,
        guard: &mut MutexGuard<'_, T>,
        mut condition: F,
    ) {
        while condition(&mut **guard) {
            self.wait(guard);
        }
    }

    /// [`Condvar::wait`] for at most `timeout`.
    pub fn wait_for<T>(
        &self,
        guard: &mut MutexGuard<'_, T>,
        timeout: Duration,
    ) -> WaitTimeoutResult {
        let held = take(guard);
        let (held, result) = self
            .inner
            .wait_timeout(held, timeout)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.inner = Some(held);
        WaitTimeoutResult(result.timed_out())
    }

    /// [`Condvar::wait`] until `deadline`.
    pub fn wait_until<T>(
        &self,
        guard: &mut MutexGuard<'_, T>,
        deadline: Instant,
    ) -> WaitTimeoutResult {
        self.wait_for(guard, deadline.saturating_duration_since(Instant::now()))
    }

    /// [`Condvar::wait_while`] for at most `timeout`; times out only if
    /// `condition` still holds when the time runs out.
    pub fn wait_while_for<T, F: FnMut(&mut T) -> bool>(
        &self,
        guard: &mut MutexGuard<'_, T>,
        condition: F,
        timeout: Duration,
    ) -> WaitTimeoutResult {
        self.wait_while_until(guard, condition, Instant::now() + timeout)
    }

    /// [`Condvar::wait_while`] until `deadline`.
    pub fn wait_while_until<T, F: FnMut(&mut T) -> bool>(
        &self,
        guard: &mut MutexGuard<'_, T>,
        mut condition: F,
        deadline: Instant,
    ) -> WaitTimeoutResult {
        while condition(&mut **guard) {
            let now = Instant::now();
            if now >= deadline {
                return WaitTimeoutResult(true);
            }
            self.wait_for(guard, deadline - now);
        }
        WaitTimeoutResult(false)
    }
}

fn take<'a, T>(guard: &mut MutexGuard<'a, T>) -> std::sync::MutexGuard<'a, T> {
    guard
        .inner
        .take()
        .expect("parking_lot: guard in use by another Condvar wait")
}
