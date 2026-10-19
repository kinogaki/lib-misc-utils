# document-features (Kinogaki)

Ours, not upstream document-features. It is published under the crates.io
name (`0.2.12+kinogaki`) for the frozen wgpu and wgpu-core.

They use it only to list their Cargo features in rustdoc. Here
`document_features!()` expands to an empty string (litrs went with it).
