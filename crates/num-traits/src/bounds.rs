//! The smallest and largest finite values of a type.

/// The smallest and largest finite values of a type.
pub trait Bounded {
    fn min_value() -> Self;
    fn max_value() -> Self;
}
