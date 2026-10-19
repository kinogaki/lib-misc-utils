//! The additive and multiplicative identities.

use core::ops::{Add, Mul};

/// The additive identity.
pub trait Zero: Sized + Add<Self, Output = Self> {
    fn zero() -> Self;
    fn is_zero(&self) -> bool;
    fn set_zero(&mut self) {
        *self = Self::zero();
    }
}

/// The multiplicative identity.
pub trait One: Sized + Mul<Self, Output = Self> {
    fn one() -> Self;
    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        *self == Self::one()
    }
}

/// `T::zero()`.
pub fn zero<T: Zero>() -> T {
    T::zero()
}

/// `T::one()`.
pub fn one<T: One>() -> T {
    T::one()
}
