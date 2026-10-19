//! The assertions compile when true (their failing forms are compile errors,
//! which the trybuild-free test suite cannot run; see the macro docs).

use std::rc::Rc;

static_assertions::assert_impl_all!(u32: Send, Sync, Copy);
static_assertions::assert_impl_all!(Vec<u8>: Send, Clone,);
static_assertions::assert_impl_all!(str: Send);
static_assertions::assert_not_impl_any!(Rc<u8>: Send, Sync);
static_assertions::assert_not_impl_any!(String: Copy);

#[test]
fn true_assertions_compile() {}
