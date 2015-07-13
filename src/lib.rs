#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]

//! Wrappers for total order on Floats.

extern crate num;
extern crate unreachable;

use std::cmp::Ordering;
use std::error::Error;
use std::ops::{Deref, DerefMut};
use std::hash::{Hash, Hasher};
use std::fmt;
use std::io;
use unreachable::unreachable;
use num::Float;

/// A wrapper around Floats providing an implementation of Ord and Hash.
///
/// NaN is sorted as *greater* than all other values and *equal*
/// to itself, in contradiction with the IEEE standard.
#[derive(PartialOrd, Debug, Default, Clone, Copy)]
pub struct OrderedFloat<T: Float>(pub T);

impl<T: Float> OrderedFloat<T> {
    /// Get the value out.
    pub fn into_inner(self) -> T {
        let OrderedFloat(val) = self;
        val
    }
}

impl<T: Float> AsRef<T> for OrderedFloat<T> {
    fn as_ref(&self) -> &T {
        let OrderedFloat(ref val) = *self;
        val
    }
}

impl<T: Float> AsMut<T> for OrderedFloat<T> {
    fn as_mut(&mut self) -> &mut T {
        let OrderedFloat(ref mut val) = *self;
        val
    }
}

impl<T: Float + PartialOrd> Ord for OrderedFloat<T> {
    fn cmp(&self, other: &OrderedFloat<T>) -> Ordering {
        match self.partial_cmp(&other) {
            Some(ordering) => ordering,
            None => {
                if self.as_ref().is_nan() {
                    if other.as_ref().is_nan() {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                } else {
                    Ordering::Less
                }
            }
        }
    }
}

impl<T: Float + PartialEq> PartialEq for OrderedFloat<T> {
    fn eq(&self, other: &OrderedFloat<T>) -> bool {
        if self.as_ref().is_nan() {
            if other.as_ref().is_nan() {
                true
            } else {
                false
            }
        } else if other.as_ref().is_nan() {
            false
        } else {
            self.as_ref() == other.as_ref()
        }
    }
}

impl<T: Float> Hash for OrderedFloat<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (man, exp, sign) = if self.as_ref().is_nan() {
            // normalize to one representation of NaN
            T::nan().integer_decode()
        } else {
            self.as_ref().integer_decode()
        };
        (man ^ exp as u64 ^ sign as u64).hash(state)
    }
}

impl<T: Float + fmt::Display> fmt::Display for OrderedFloat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl Into<f32> for OrderedFloat<f32> {
    fn into(self) -> f32 {
        self.into_inner()
    }
}

impl Into<f64> for OrderedFloat<f64> {
    fn into(self) -> f64 {
        self.into_inner()
    }
}

impl<T: Float> From<T> for OrderedFloat<T> {
    fn from(val: T) -> Self {
        OrderedFloat(val)
    }
}

impl<T: Float> Deref for OrderedFloat<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: Float> DerefMut for OrderedFloat<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: Float + PartialEq> Eq for OrderedFloat<T> { }

/// A wrapper around Floats providing an implementation of Ord and Hash.
///
/// A NaN value cannot be stored in this type.
#[derive(PartialOrd, PartialEq, Debug, Default, Clone, Copy)]
pub struct NotNaN<T: Float>(T);

impl<T: Float> NotNaN<T> {
    /// Create a NotNaN value.
    ///
    /// Returns Err if val is NaN
    pub fn new(val: T) -> Result<Self, FloatIsNaN> {
        match val {
            ref val if val.is_nan() => Err(FloatIsNaN),
            val => Ok(NotNaN(val)),
        }
    }

    /// Create a NotNaN value from a value that is guaranteed to not be NaN
    ///
    /// Behaviour is undefined if `val` is NaN
    pub unsafe fn unchecked_new(val: T) -> Self {
        debug_assert!(!val.is_nan());
        NotNaN(val)
    }

    /// Get the value out.
    pub fn into_inner(self) -> T {
        let NotNaN(val) = self;
        val
    }
}

impl<T: Float> AsRef<T> for NotNaN<T> {
    fn as_ref(&self) -> &T {
        let NotNaN(ref val) = *self;
        val
    }
}

impl<T: Float + PartialOrd> Ord for NotNaN<T> {
    fn cmp(&self, other: &NotNaN<T>) -> Ordering {
        match self.partial_cmp(&other) {
            Some(ord) => ord,
            None => unsafe { unreachable() },
        }
    }
}

impl<T: Float> Hash for NotNaN<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (man, exp, sign) = self.as_ref().integer_decode();
        (man ^ exp as u64 ^ sign as u64).hash(state)
    }
}

impl<T: Float + fmt::Display> fmt::Display for NotNaN<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl Into<f32> for NotNaN<f32> {
    fn into(self) -> f32 {
        self.into_inner()
    }
}

impl Into<f64> for NotNaN<f64> {
    fn into(self) -> f64 {
        self.into_inner()
    }
}

impl<T: Float> From<T> for NotNaN<T> {
    fn from(v: T) -> Self {
        assert!(!v.is_nan());
        NotNaN(v)
    }
}

impl<T: Float> Deref for NotNaN<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: Float + PartialEq> Eq for NotNaN<T> {}

/// An error indicating an attempt to construct NotNaN from a NaN
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FloatIsNaN;

impl Error for FloatIsNaN {
    fn description(&self) -> &str {
        return "NotNaN constructed with NaN"
    }
}

impl fmt::Display for FloatIsNaN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl Into<io::Error> for FloatIsNaN {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidInput, self)
    }
}
