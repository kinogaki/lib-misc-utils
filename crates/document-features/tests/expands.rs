#![allow(clippy::empty_docs)] // what wgpu writes, with our empty expansion
#![doc = document_features::document_features!()]

#[test]
fn expands_to_an_empty_string() {
    assert_eq!(document_features::document_features!(), "");
    assert_eq!(
        document_features::document_features!(feature_label = "{feature}"),
        ""
    );
}
