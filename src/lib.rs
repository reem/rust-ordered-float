#![license = "MIT"]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(globs, phase)]

//! Wrappers for total order on Floats.

#[cfg(test)] #[phase(plugin)] extern crate stainless;

use std::num::Float;

/// A wrapper around Floats providing an implementation of Ord.
///
/// NaN is sorted as *greater* than all other values and *equal*
/// to itself, in contradiction with the IEEE standard.
#[deriving(PartialOrd, Show, Clone)]
pub struct OrderedFloat<T: Float>(pub T);

impl<T: Float> OrderedFloat<T> {
    /// Get the value out.
    pub fn unwrap(self) -> T {
        let OrderedFloat(val) = self;
        val
    }
}

impl<T: Float + PartialOrd> Ord for OrderedFloat<T> {
    fn cmp(&self, other: &OrderedFloat<T>) -> Ordering {
        match self.unwrap().partial_cmp(&other.unwrap()) {
            Some(ordering) => ordering,
            None => {
                if self.unwrap().is_nan() {
                    if other.unwrap().is_nan() {
                        Equal
                    } else {
                        Greater
                    }
                } else {
                    Less
                }
            }
        }
    }
}

impl<T: Float + PartialEq> PartialEq for OrderedFloat<T> {
    fn eq(&self, other: &OrderedFloat<T>) -> bool {
        if self.unwrap().is_nan() {
            if other.unwrap().is_nan() {
                true
            } else {
                false
            }
        } else if other.unwrap().is_nan() {
            false
        } else {
            self.unwrap() == other.unwrap()
        }

    }
}

impl<T: Float + PartialEq> Eq for OrderedFloat<T> { }

/// A wrapper around Floats providing an implementation of Ord.
///
/// If NaN is encountered becuase NotNaN was manually constructed
/// with a NaN value, this will panic.
#[deriving(PartialOrd, Show, Clone)]
pub struct NotNaN<T: Float>(pub T);

impl<T: Float> NotNaN<T> {
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
    pub fn unwrap(self) -> T {
        let NotNaN(val) = self;
        val
    }
}

impl<T: Float + PartialOrd> Ord for NotNaN<T> {
    fn cmp(&self, other: &NotNaN<T>) -> Ordering {
        self.unwrap()
            .partial_cmp(&other.unwrap())
            .expect("NaN encountered in NotNaN comparison.")
    }
}

impl<T: Float + PartialEq> PartialEq for NotNaN<T> {
    fn eq(&self, other: &NotNaN<T>) -> bool {
        if self.unwrap().is_nan() || other.unwrap().is_nan() {
            panic!("NaN encountered in NotNaN comparison.")
        } else {
            self.unwrap() == other.unwrap()
        }
    }
}

impl<T: Float + PartialEq> Eq for NotNaN<T> {}

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use std::num::Float;

    describe! ordered_float32 {
        it "should compare regular floats" {
            assert_eq!(OrderedFloat(7.0f32).cmp(&OrderedFloat(7.0)), Equal)
            assert_eq!(OrderedFloat(8.0f32).cmp(&OrderedFloat(7.0)), Greater)
            assert_eq!(OrderedFloat(4.0f32).cmp(&OrderedFloat(7.0)), Less)
        }

        it "should compare NaN" {
            let f32_nan: f32 = Float::nan();
            assert_eq!(OrderedFloat(f32_nan).cmp(&OrderedFloat(Float::nan())), Equal);
            assert_eq!(OrderedFloat(Float::nan()).cmp(&OrderedFloat(-100000.0f32)), Greater);
            assert_eq!(OrderedFloat(-100.0f32).cmp(&OrderedFloat(Float::nan())), Less);
        }
    }

    describe! ordered_float64 {
        it "should compare regular floats" {
            assert_eq!(OrderedFloat(7.0f64).cmp(&OrderedFloat(7.0)), Equal)
            assert_eq!(OrderedFloat(8.0f64).cmp(&OrderedFloat(7.0)), Greater)
            assert_eq!(OrderedFloat(4.0f64).cmp(&OrderedFloat(7.0)), Less)
        }

        it "should compare NaN" {
            let f64_nan: f64 = Float::nan();
            assert_eq!(OrderedFloat(f64_nan).cmp(&OrderedFloat(Float::nan())), Equal);
            assert_eq!(OrderedFloat(Float::nan()).cmp(&OrderedFloat(-100000.0f64)), Greater);
            assert_eq!(OrderedFloat(-100.0f64).cmp(&OrderedFloat(Float::nan())), Less);
        }
    }

    describe! not_nan32 {
        it "should compare regular floats" {
            assert_eq!(NotNaN(7.0f32).cmp(&NotNaN(7.0)), Equal)
            assert_eq!(NotNaN(8.0f32).cmp(&NotNaN(7.0)), Greater)
            assert_eq!(NotNaN(4.0f32).cmp(&NotNaN(7.0)), Less)
        }

        failing "should fail when comparing NaN to NaN" {
            let f32_nan: f32 = Float::nan();
            NotNaN(f32_nan).cmp(&NotNaN(Float::nan()));
        }

        failing "should fail when comparing NaN to a regular float" {
            NotNaN(Float::nan()).cmp(&NotNaN(7.0f32));
        }

        failing "should fail when comparing a regular float to NaN" {
            NotNaN(7.0f32).cmp(&NotNaN(Float::nan()));
        }

        failing "should fail when constructing NotNaN with NaN" {
            let f32_nan: f32 = Float::nan();
            NotNaN::new(f32_nan);
        }
    }

    describe! not_nan64 {
        it "should compare regular floats" {
            assert_eq!(NotNaN(7.0f64).cmp(&NotNaN(7.0)), Equal)
            assert_eq!(NotNaN(8.0f64).cmp(&NotNaN(7.0)), Greater)
            assert_eq!(NotNaN(4.0f64).cmp(&NotNaN(7.0)), Less)
        }

        failing "should panic when comparing NaN to NaN" {
            let f64_nan: f64 = Float::nan();
            NotNaN(f64_nan).cmp(&NotNaN(Float::nan()));
        }

        failing "should fail when comparing NaN to a regular float" {
            NotNaN(Float::nan()).cmp(&NotNaN(7.0f64));
        }

        failing "should fail when comparing a regular float to NaN" {
            NotNaN(7.0f64).cmp(&NotNaN(Float::nan()));
        }

        failing "should fail when constructing NotNaN with NaN" {
            let f64_nan: f64 = Float::nan();
            NotNaN::new(f64_nan);
        }
    }
}

