//! Kinogaki's own `document-features`, published under the crates.io name so
//! the frozen wgpu and wgpu-core resolve to it through `[patch.crates-io]`.
//! It is not upstream document-features: those crates use it only in
//! `#![doc = document_features::document_features!()]` to list their Cargo
//! features in rustdoc, which we never build for them. The macro expands to
//! an empty string.

/// An empty feature list (accepts and ignores upstream's options).
#[macro_export]
macro_rules! document_features {
    ($($options:tt)*) => {
        ""
    };
}
