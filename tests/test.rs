#![feature(plugin)]
#![plugin(stainless)]

extern crate ordered_float;
extern crate num;

pub use ordered_float::*;
pub use num::Float;
pub use std::cmp::Ordering::*;

describe! ordered_float32 {
    it "should compare regular floats" {
        assert_eq!(OrderedFloat(7.0f32).cmp(&OrderedFloat(7.0)), Equal);
        assert_eq!(OrderedFloat(8.0f32).cmp(&OrderedFloat(7.0)), Greater);
        assert_eq!(OrderedFloat(4.0f32).cmp(&OrderedFloat(7.0)), Less);
    }

    it "should compare NaN" {
        let f32_nan: f32 = Float::nan();
        assert_eq!(OrderedFloat(f32_nan).cmp(&OrderedFloat(Float::nan())), Equal);
        assert_eq!(OrderedFloat(f32_nan).cmp(&OrderedFloat(-100000.0f32)), Greater);
        assert_eq!(OrderedFloat(-100.0f32).cmp(&OrderedFloat(Float::nan())), Less);
    }
}

describe! ordered_float64 {
    it "should compare regular floats" {
        assert_eq!(OrderedFloat(7.0f64).cmp(&OrderedFloat(7.0)), Equal);
        assert_eq!(OrderedFloat(8.0f64).cmp(&OrderedFloat(7.0)), Greater);
        assert_eq!(OrderedFloat(4.0f64).cmp(&OrderedFloat(7.0)), Less);
    }

    it "should compare NaN" {
        let f64_nan: f64 = Float::nan();
        assert_eq!(OrderedFloat(f64_nan).cmp(&OrderedFloat(Float::nan())), Equal);
        assert_eq!(OrderedFloat(f64_nan).cmp(&OrderedFloat(-100000.0f64)), Greater);
        assert_eq!(OrderedFloat(-100.0f64).cmp(&OrderedFloat(Float::nan())), Less);
    }
}

describe! not_nan32 {
    it "should compare regular floats" {
        assert_eq!(NotNaN::from(7.0f32).cmp(&NotNaN::from(7.0)), Equal);
        assert_eq!(NotNaN::from(8.0f32).cmp(&NotNaN::from(7.0)), Greater);
        assert_eq!(NotNaN::from(4.0f32).cmp(&NotNaN::from(7.0)), Less);
    }

    it "should fail when constructing NotNaN with NaN" {
        let f32_nan: f32 = Float::nan();
        assert!(NotNaN::new(f32_nan).is_err());
    }
}

describe! not_nan64 {
    it "should compare regular floats" {
        assert_eq!(NotNaN::from(7.0f64).cmp(&NotNaN::from(7.0)), Equal);
        assert_eq!(NotNaN::from(8.0f64).cmp(&NotNaN::from(7.0)), Greater);
        assert_eq!(NotNaN::from(4.0f64).cmp(&NotNaN::from(7.0)), Less);
    }

    it "should fail when constructing NotNaN with NaN" {
        let f64_nan: f64 = Float::nan();
        assert!(NotNaN::new(f64_nan).is_err());
    }
}
