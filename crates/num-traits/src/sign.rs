//! Sign queries for signed numbers.

use crate::Num;

/// `abs`, `signum` and sign tests for signed numbers.
pub trait Signed: Num + core::ops::Neg<Output = Self> {
    fn abs(&self) -> Self;
    fn signum(&self) -> Self;
    fn is_positive(&self) -> bool;
    fn is_negative(&self) -> bool;
}
