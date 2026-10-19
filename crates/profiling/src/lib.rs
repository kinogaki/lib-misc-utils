//! Kinogaki's own `profiling`, published under the crates.io name so the
//! frozen wgpu, wgpu-core and wgpu-hal resolve to it through
//! `[patch.crates-io]`. It is not upstream profiling: those crates mark
//! regions with `profiling::scope!`, and with no profiler backend (as they
//! build here) the marks expand to nothing.

/// Mark the rest of the enclosing block as a named profiling scope (no-op).
#[macro_export]
macro_rules! scope {
    ($name:expr) => {};
    ($name:expr, $data:expr) => {};
}

/// Mark the end of a frame (no-op).
#[macro_export]
macro_rules! finish_frame {
    () => {};
}

/// Name the current thread for the profiler (no-op).
#[macro_export]
macro_rules! register_thread {
    () => {};
    ($name:expr) => {};
}
