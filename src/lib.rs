#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]
#![feature(globs, phase)]

//! Wrappers for total order on f64s.

#[cfg(test)] #[phase(plugin)] extern crate stainless;

use std::num::Float;

/// A wrapper around f64 providing an implementation of Ord.
///
/// NaN is sorted as *greater* than all other values and *equal*
/// to itself, in contradiction with the IEEE standard.
#[deriving(PartialOrd, Show, Clone)]
pub struct OrderedFloat(pub f64);

impl OrderedFloat {
    /// Get the value out.
    pub fn unwrap(self) -> f64 {
        let OrderedFloat(val) = self;
        val
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &OrderedFloat) -> Ordering {
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

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &OrderedFloat) -> bool {
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

impl Eq for OrderedFloat { }

/// A wrapper around f64 providing an implementation of Ord.
///
/// If NaN is encountered becuase NotNaN was manually constructed
/// with a NaN value, this will fail.
#[deriving(PartialOrd, Show, Clone)]
pub struct NotNaN(pub f64);

impl NotNaN {
    /// Creat a NotNaN value.
    ///
    /// ## Failure
    ///
    /// Fails if the val is NaN
    pub fn new(val: f64) -> NotNaN {
        if val.is_nan() { fail!("NaN encountered in NotNaN construction.") }
        NotNaN(val)
    }

    /// Get the value out.
    pub fn unwrap(self) -> f64 {
        let NotNaN(val) = self;
        val
    }
}

impl Ord for NotNaN {
    fn cmp(&self, other: &NotNaN) -> Ordering {
        self.unwrap()
            .partial_cmp(&other.unwrap())
            .expect("NaN encountered in NotNaN comparison.")
    }
}

impl PartialEq for NotNaN {
    fn eq(&self, other: &NotNaN) -> bool {
        if self.unwrap().is_nan() || other.unwrap().is_nan() {
            fail!("NaN encountered in NotNaN comparison.")
        } else {
            self.unwrap() == other.unwrap()
        }
    }
}

impl Eq for NotNaN {}

#[cfg(test)]
mod tests {
    pub use super::*;

    describe! ordered_float {
        it "should compare regular floats" {
            assert_eq!(OrderedFloat(7.0).cmp(&OrderedFloat(7.0)), Equal)
            assert_eq!(OrderedFloat(8.0).cmp(&OrderedFloat(7.0)), Greater)
            assert_eq!(OrderedFloat(4.0).cmp(&OrderedFloat(7.0)), Less)
        }

        it "should compare NaN" {
            assert_eq!(OrderedFloat(Float::nan()).cmp(&OrderedFloat(Float::nan())), Equal);
            assert_eq!(OrderedFloat(Float::nan()).cmp(&OrderedFloat(-100000.0)), Greater);
            assert_eq!(OrderedFloat(-100.0).cmp(&OrderedFloat(Float::nan())), Less);
        }
    }

    describe! not_nan {
        it "should compare regular floats" {
            assert_eq!(NotNaN(7.0).cmp(&NotNaN(7.0)), Equal)
            assert_eq!(NotNaN(8.0).cmp(&NotNaN(7.0)), Greater)
            assert_eq!(NotNaN(4.0).cmp(&NotNaN(7.0)), Less)
        }

        failing "should fail when comparing NaN to NaN" {
            NotNaN(Float::nan()).cmp(&NotNaN(Float::nan()));
        }

        failing "should fail when comparing NaN to a regular float" {
            NotNaN(Float::nan()).cmp(&NotNaN(7.0));
        }

        failing "should fail when comparing a regular float to NaN" {
            NotNaN(7.0).cmp(&NotNaN(Float::nan()));
        }

        failing "should fail when constructing NotNaN with NaN" {
            NotNaN::new(Float::nan());
        }
    }
}

