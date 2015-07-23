#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]

//! Wrappers for total order on Floats.

extern crate num;

use num::Float;
use std::cmp::Ordering;

/// A wrapper around Floats providing an implementation of Ord.
///
/// NaN is sorted as *greater* than all other values and *equal*
/// to itself, in contradiction with the IEEE standard.
#[derive(PartialOrd, Debug, Clone, Hash, Copy)]
pub struct OrderedFloat<T: Float + Copy>(pub T);

impl<T: Float + Copy> AsRef<T> for OrderedFloat<T> {
    /// Get the value out.
    fn as_ref(&self) -> &T {
        let OrderedFloat(ref val) = *self;
        val
    }
}

impl<T: Float + Copy + PartialOrd> Ord for OrderedFloat<T> {
    fn cmp(&self, other: &OrderedFloat<T>) -> Ordering {
        match self.as_ref().partial_cmp(&other.as_ref()) {
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

impl<T: Float + Copy + PartialEq> PartialEq for OrderedFloat<T> {
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

impl<T: Float + Copy + PartialEq> Eq for OrderedFloat<T> { }

/// A wrapper around Floats providing an implementation of Ord.
///
/// If NaN is encountered becuase NotNaN was manually constructed
/// with a NaN value, this will panic.
#[derive(PartialOrd, Debug, Clone, Hash, Copy)]
pub struct NotNaN<T: Float + Copy>(pub T);

impl<T: Float + Copy> NotNaN<T> {
    /// Create a NotNaN value.
    ///
    /// ## Panics
    ///
    /// Panics if the val is NaN
    pub fn new(val: T) -> NotNaN<T> {
        if val.is_nan() { panic!("NaN encountered in NotNaN construction.") }
        NotNaN(val)
    }

    /// Get the value out.
    pub fn as_ref(self) -> T {
        let NotNaN(val) = self;
        val
    }
}

impl<T: Float + Copy + PartialOrd> Ord for NotNaN<T> {
    fn cmp(&self, other: &NotNaN<T>) -> Ordering {
        self.as_ref()
            .partial_cmp(&other.as_ref())
            .expect("NaN encountered in NotNaN comparison.")
    }
}

impl<T: Float + Copy + PartialEq> PartialEq for NotNaN<T> {
    fn eq(&self, other: &NotNaN<T>) -> bool {
        if self.as_ref().is_nan() || other.as_ref().is_nan() {
            panic!("NaN encountered in NotNaN comparison.")
        } else {
            self.as_ref() == other.as_ref()
        }
    }
}

impl<T: Float + Copy + PartialEq> Eq for NotNaN<T> {}

