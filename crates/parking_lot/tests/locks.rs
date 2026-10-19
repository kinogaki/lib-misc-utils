//! The lock semantics wgpu-core and wgpu-hal rely on.

use parking_lot::{Condvar, Mutex, RwLock, RwLockReadGuard};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn readers_share_and_writers_exclude() {
    // Two fields written in two steps by writers; readers must never see them differ.
    let lock = Arc::new(RwLock::new((0u64, 0u64)));
    let stop = Arc::new(AtomicBool::new(false));
    let mut threads = Vec::new();
    for _ in 0..4 {
        let (lock, stop) = (lock.clone(), stop.clone());
        threads.push(thread::spawn(move || {
            let mut seen = 0u64;
            while !stop.load(Ordering::Relaxed) {
                let pair = lock.read();
                assert_eq!(pair.0, pair.1, "a reader saw a half-done write");
                seen += 1;
            }
            seen
        }));
    }
    let writers: Vec<_> = (0..4)
        .map(|_| {
            let lock = lock.clone();
            thread::spawn(move || {
                for _ in 0..20_000 {
                    let mut pair = lock.write();
                    pair.0 += 1;
                    std::hint::spin_loop();
                    pair.1 += 1;
                }
            })
        })
        .collect();
    for writer in writers {
        writer.join().unwrap();
    }
    stop.store(true, Ordering::Relaxed);
    for reader in threads {
        assert!(reader.join().unwrap() > 0);
    }
    assert_eq!(*lock.read(), (80_000, 80_000));
}

#[test]
fn many_readers_hold_at_once() {
    let lock = Arc::new(RwLock::new(()));
    let inside = Arc::new(AtomicUsize::new(0));
    let most = Arc::new(AtomicUsize::new(0));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let (lock, inside, most) = (lock.clone(), inside.clone(), most.clone());
            thread::spawn(move || {
                let _guard = lock.read();
                let now = inside.fetch_add(1, Ordering::SeqCst) + 1;
                most.fetch_max(now, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(50));
                inside.fetch_sub(1, Ordering::SeqCst);
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    assert!(most.load(Ordering::SeqCst) > 1, "readers did not overlap");
}

#[test]
fn recursive_reads_without_a_waiting_writer() {
    let lock = RwLock::new(5);
    let a = lock.read();
    let b = lock.read();
    assert_eq!(*a + *b, 10);
    assert!(lock.try_write().is_none());
    drop((a, b));
    assert!(lock.try_write().is_some());
}

/// Once a writer waits, new readers wait behind it (no writer starvation),
/// and the writer goes first.
#[test]
fn a_waiting_writer_goes_before_new_readers() {
    let lock = Arc::new(RwLock::new(Vec::new()));
    let first = lock.read();
    let writer = {
        let lock = lock.clone();
        thread::spawn(move || lock.write().push("writer"))
    };
    // Wait until the writer has announced itself: then try_read refuses.
    let deadline = Instant::now() + Duration::from_secs(10);
    while lock.try_read().is_some() {
        assert!(
            Instant::now() < deadline,
            "the writer never started waiting"
        );
        thread::yield_now();
    }
    let reader = {
        let lock = lock.clone();
        thread::spawn(move || {
            let order = lock.read();
            order.first().copied()
        })
    };
    thread::sleep(Duration::from_millis(50));
    drop(first);
    writer.join().unwrap();
    assert_eq!(reader.join().unwrap(), Some("writer"));
}

/// Continuous overlapping readers on several threads cannot keep a writer out.
#[test]
fn writers_are_not_starved_by_readers() {
    let lock = Arc::new(RwLock::new(0u32));
    let stop = Arc::new(AtomicBool::new(false));
    let readers: Vec<_> = (0..6)
        .map(|_| {
            let (lock, stop) = (lock.clone(), stop.clone());
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let _guard = lock.read();
                    thread::sleep(Duration::from_micros(200));
                }
            })
        })
        .collect();
    thread::sleep(Duration::from_millis(20));
    let start = Instant::now();
    for _ in 0..50 {
        *lock.write() += 1;
    }
    let took = start.elapsed();
    stop.store(true, Ordering::Relaxed);
    for r in readers {
        r.join().unwrap();
    }
    assert_eq!(*lock.read(), 50);
    assert!(
        took < Duration::from_secs(2),
        "50 writes took {took:?} against readers"
    );
}

#[test]
fn forgotten_read_is_released_by_force_unlock() {
    let lock = RwLock::new(());
    let guard = lock.read();
    std::mem::forget::<RwLockReadGuard<'_, ()>>(guard);
    assert!(lock.is_locked());
    assert!(lock.try_write().is_none());
    // SAFETY: the read above is held and its guard was forgotten.
    #[allow(unsafe_code)]
    unsafe {
        lock.force_unlock_read()
    };
    assert!(!lock.is_locked());
    assert!(lock.try_write().is_some());
}

#[test]
fn a_panic_does_not_poison() {
    let mutex = Arc::new(Mutex::new(1));
    let rw = Arc::new(RwLock::new(1));
    let (m, r) = (mutex.clone(), rw.clone());
    let _ = thread::spawn(move || {
        let _a = m.lock();
        let _b = r.write();
        panic!("while holding both");
    })
    .join();
    *mutex.lock() += 1;
    *rw.write() += 1;
    assert_eq!((*mutex.lock(), *rw.read()), (2, 2));
}

#[test]
fn mutex_basics() {
    let mutex = Mutex::new(vec![1]);
    {
        let mut guard = mutex.lock();
        guard.push(2);
        assert!(mutex.is_locked());
        assert!(mutex.try_lock().is_none());
    }
    assert!(!mutex.is_locked());
    assert_eq!(mutex.into_inner(), vec![1, 2]);
}

#[test]
fn condvar_waits_and_times_out() {
    let pair = Arc::new((Mutex::new(0u32), Condvar::new()));
    // Timeout when nobody signals.
    {
        let (mutex, condvar) = &*pair;
        let mut guard = mutex.lock();
        let start = Instant::now();
        let result = condvar.wait_while_for(&mut guard, |v| *v < 1, Duration::from_millis(30));
        assert!(result.timed_out());
        assert!(start.elapsed() >= Duration::from_millis(30));
        assert_eq!(*guard, 0);
    }
    // A signalled value ends the wait (as wgpu-hal's Metal fence wait does).
    let signaller = {
        let pair = pair.clone();
        thread::spawn(move || {
            for _ in 0..3 {
                thread::sleep(Duration::from_millis(5));
                *pair.0.lock() += 1;
                pair.1.notify_all();
            }
        })
    };
    let (mutex, condvar) = &*pair;
    let mut guard = mutex.lock();
    let result = condvar.wait_while_for(&mut guard, |v| *v < 3, Duration::from_secs(10));
    assert!(!result.timed_out());
    assert_eq!(*guard, 3);
    drop(guard);
    signaller.join().unwrap();
    let mut guard = mutex.lock();
    condvar.wait_while(&mut guard, |v| *v < 3);
    let r = condvar.wait_for(&mut guard, Duration::from_millis(1));
    assert!(r.timed_out());
}

#[test]
fn rwlock_stress_with_try_and_mixed_ops() {
    let lock = Arc::new(RwLock::new(0i64));
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let lock = lock.clone();
            thread::spawn(move || {
                for n in 0..20_000 {
                    match (i + n) % 5 {
                        0 => *lock.write() += 1,
                        1 => {
                            if let Some(mut g) = lock.try_write() {
                                *g += 1;
                                *g -= 1;
                            }
                        }
                        2 => {
                            let _ = lock.try_read().map(|g| *g);
                        }
                        _ => {
                            let _ = *lock.read();
                        }
                    }
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(*lock.read(), 8 * 20_000 / 5);
    assert!(!lock.is_locked());
}
