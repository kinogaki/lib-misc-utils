# profiling (Kinogaki)

Ours, not upstream profiling. It is published under the crates.io name
(`1.0.18+kinogaki`) for the frozen wgpu, wgpu-core and wgpu-hal.

`scope!`, `finish_frame!` and `register_thread!` expand to nothing, which is
what they are with no profiler backend.
