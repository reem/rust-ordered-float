#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]

//! Crate comment goes here

/// A wrapper around f64 providing an implementation of Ord.
///
/// NaN is sorted as *smaller* than all other values and *equal*
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
                        Less
                    }
                } else {
                    Greater
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
        match self.partial_cmp(other) {
            Some(ordering) => ordering,
            None => fail!("NaN encountered in NotNaN comparison.")
        }
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

