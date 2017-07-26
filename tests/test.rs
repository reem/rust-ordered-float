#![feature(plugin)]
#![plugin(stainless)]

extern crate ordered_float;
extern crate num_traits;

pub use ordered_float::*;
pub use num_traits::{Bounded, Float, FromPrimitive, Num, One, Signed, ToPrimitive, Zero};
pub use std::cmp::Ordering::*;
pub use std::{f32, f64, panic};

pub use std::collections::HashSet;
pub use std::collections::hash_map::RandomState;
pub use std::hash::*;

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
    
    it "should calculate correctly" {
        assert_eq!(*(NotNaN::from(5.0f32) + NotNaN::from(4.0f32)), 5.0f32 + 4.0f32);
        assert_eq!(*(NotNaN::from(5.0f32) + 4.0f32), 5.0f32 + 4.0f32);
        assert_eq!(*(NotNaN::from(5.0f32) - NotNaN::from(4.0f32)), 5.0f32 - 4.0f32);
        assert_eq!(*(NotNaN::from(5.0f32) - 4.0f32), 5.0f32 - 4.0f32);
        assert_eq!(*(NotNaN::from(5.0f32) * NotNaN::from(4.0f32)), 5.0f32 * 4.0f32);
        assert_eq!(*(NotNaN::from(5.0f32) * 4.0f32), 5.0f32 * 4.0f32);
        assert_eq!(*(NotNaN::from(8.0f32) / NotNaN::from(4.0f32)), 8.0f32 / 4.0f32);
        assert_eq!(*(NotNaN::from(8.0f32) / 4.0f32), 8.0f32 / 4.0f32);
        assert_eq!(*(NotNaN::from(8.0f32) % NotNaN::from(4.0f32)), 8.0f32 % 4.0f32);
        assert_eq!(*(NotNaN::from(8.0f32) % 4.0f32), 8.0f32 % 4.0f32);
        assert_eq!(*(-NotNaN::from(1.0f32)), -1.0f32);
        
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f32) + f32::NAN}).is_err());
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f32) - f32::NAN}).is_err());
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f32) * f32::NAN}).is_err());
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f32) / f32::NAN}).is_err());
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f32) % f32::NAN}).is_err());
        
        let mut number = NotNaN::from(5.0f32);
        number += NotNaN::from(4.0f32);
        assert_eq!(*number, 9.0f32);
        number -= NotNaN::from(4.0f32);
        assert_eq!(*number, 5.0f32);
        number *= NotNaN::from(4.0f32);
        assert_eq!(*number, 20.0f32);
        number /= NotNaN::from(4.0f32);
        assert_eq!(*number, 5.0f32);
        number %= NotNaN::from(4.0f32);
        assert_eq!(*number, 1.0f32);
        
        number = NotNaN::from(5.0f32);
        number += 4.0f32;
        assert_eq!(*number, 9.0f32);
        number -= 4.0f32;
        assert_eq!(*number, 5.0f32);
        number *= 4.0f32;
        assert_eq!(*number, 20.0f32);
        number /= 4.0f32;
        assert_eq!(*number, 5.0f32);
        number %= 4.0f32;
        assert_eq!(*number, 1.0f32);
        
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f32); tmp += f32::NAN;}).is_err());
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f32); tmp -= f32::NAN;}).is_err());
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f32); tmp *= f32::NAN;}).is_err());
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f32); tmp /= f32::NAN;}).is_err());
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f32); tmp %= f32::NAN;}).is_err());
    }

    it "should implement Zero" {
        assert_eq!(NotNaN::<f32>::zero(), NotNaN::from(0.0f32));
        assert!(NotNaN::<f32>::zero().is_zero());
    }

    it "should implement One" {
        assert_eq!(NotNaN::<f32>::one(), NotNaN::from(1.0f32))
    }

    it "should implement Bounded" {
        assert_eq!(NotNaN::<f32>::min_value(), NotNaN::from(<f32 as Bounded>::min_value()));
        assert_eq!(NotNaN::<f32>::max_value(), NotNaN::from(<f32 as Bounded>::max_value()));
    }

    it "should implement FromPrimitive" {
        assert_eq!(NotNaN::<f32>::from_i8(42i8), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_u8(42u8), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_i16(42i16), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_u16(42u16), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_i32(42i32), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_u32(42u32), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_i64(42i64), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_u64(42u64), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_isize(42isize), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_usize(42usize), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_f32(42f32), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_f32(42f32), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_f64(42f64), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_f64(42f64), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f32>::from_f32(Float::nan()), None);
        assert_eq!(NotNaN::<f32>::from_f64(Float::nan()), None);
    }

    it "should implement ToPrimitive" {
        let x = NotNaN::from(42.0f32);
        assert_eq!(x.to_u8(), Some(42u8));
        assert_eq!(x.to_i8(), Some(42i8));
        assert_eq!(x.to_u16(), Some(42u16));
        assert_eq!(x.to_i16(), Some(42i16));
        assert_eq!(x.to_u32(), Some(42u32));
        assert_eq!(x.to_i32(), Some(42i32));
        assert_eq!(x.to_u64(), Some(42u64));
        assert_eq!(x.to_i64(), Some(42i64));
        assert_eq!(x.to_usize(), Some(42usize));
        assert_eq!(x.to_isize(), Some(42isize));
        assert_eq!(x.to_f32(), Some(42f32));
        assert_eq!(x.to_f32(), Some(42f32));
        assert_eq!(x.to_f64(), Some(42f64));
        assert_eq!(x.to_f64(), Some(42f64));
    }

    it "should implement Num" {
        assert_eq!(NotNaN::<f32>::from_str_radix("42.0", 10).unwrap(), NotNaN::from(42.0f32));
        assert!(NotNaN::<f32>::from_str_radix("NaN", 10).is_err());
    }

    it "should implement Signed" {
        assert_eq!(NotNaN::from(42f32).abs(), NotNaN::from(42f32));
        assert_eq!(NotNaN::from(-42f32).abs(), NotNaN::from(42f32));

        assert_eq!(NotNaN::from(50f32).abs_sub(&NotNaN::from(8f32)), NotNaN::from(42f32));
        assert_eq!(NotNaN::from(8f32).abs_sub(&NotNaN::from(50f32)), NotNaN::from(0f32));
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
    
    it "should calculate correctly" {
        assert_eq!(*(NotNaN::from(5.0f64) + NotNaN::from(4.0f64)), 5.0f64 + 4.0f64);
        assert_eq!(*(NotNaN::from(5.0f64) + 4.0f64), 5.0f64 + 4.0f64);
        assert_eq!(*(NotNaN::from(5.0f64) - NotNaN::from(4.0f64)), 5.0f64 - 4.0f64);
        assert_eq!(*(NotNaN::from(5.0f64) - 4.0f64), 5.0f64 - 4.0f64);
        assert_eq!(*(NotNaN::from(5.0f64) * NotNaN::from(4.0f64)), 5.0f64 * 4.0f64);
        assert_eq!(*(NotNaN::from(5.0f64) * 4.0f64), 5.0f64 * 4.0f64);
        assert_eq!(*(NotNaN::from(8.0f64) / NotNaN::from(4.0f64)), 8.0f64 / 4.0f64);
        assert_eq!(*(NotNaN::from(8.0f64) / 4.0f64), 8.0f64 / 4.0f64);
        assert_eq!(*(NotNaN::from(8.0f64) % NotNaN::from(4.0f64)), 8.0f64 % 4.0f64);
        assert_eq!(*(NotNaN::from(8.0f64) % 4.0f64), 8.0f64 % 4.0f64);
        assert_eq!(*(-NotNaN::from(1.0f64)), -1.0f64);
        
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f64) + f64::NAN}).is_err());
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f64) - f64::NAN}).is_err());
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f64) * f64::NAN}).is_err());
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f64) / f64::NAN}).is_err());
        assert!(panic::catch_unwind(|| {NotNaN::from(0.0f64) % f64::NAN}).is_err());
        
        let mut number = NotNaN::from(5.0f64);
        number += NotNaN::from(4.0f64);
        assert_eq!(*number, 9.0f64);
        number -= NotNaN::from(4.0f64);
        assert_eq!(*number, 5.0f64);
        number *= NotNaN::from(4.0f64);
        assert_eq!(*number, 20.0f64);
        number /= NotNaN::from(4.0f64);
        assert_eq!(*number, 5.0f64);
        number %= NotNaN::from(4.0f64);
        assert_eq!(*number, 1.0f64);
        
        number = NotNaN::from(5.0f64);
        number += 4.0f64;
        assert_eq!(*number, 9.0f64);
        number -= 4.0f64;
        assert_eq!(*number, 5.0f64);
        number *= 4.0f64;
        assert_eq!(*number, 20.0f64);
        number /= 4.0f64;
        assert_eq!(*number, 5.0f64);
        number %= 4.0f64;
        assert_eq!(*number, 1.0f64);
        
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f64); tmp += f64::NAN;}).is_err());
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f64); tmp -= f64::NAN;}).is_err());
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f64); tmp *= f64::NAN;}).is_err());
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f64); tmp /= f64::NAN;}).is_err());
        assert!(panic::catch_unwind(|| {let mut tmp = NotNaN::from(0.0f64); tmp %= f64::NAN;}).is_err());
    }

    it "should implement Zero" {
        assert_eq!(NotNaN::<f64>::zero(), NotNaN::from(0.0f64));
        assert!(NotNaN::<f64>::zero().is_zero());
    }

    it "should implement One" {
        assert_eq!(NotNaN::<f64>::one(), NotNaN::from(1.0f64))
    }

    it "should implement Bounded" {
        assert_eq!(NotNaN::<f64>::min_value(), NotNaN::from(<f64 as Bounded>::min_value()));
        assert_eq!(NotNaN::<f64>::max_value(), NotNaN::from(<f64 as Bounded>::max_value()));
    }

    it "should implement FromPrimitive" {
        assert_eq!(NotNaN::<f64>::from_i8(42i8), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_u8(42u8), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_i16(42i16), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_u16(42u16), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_i32(42i32), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_u32(42u32), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_i64(42i64), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_u64(42u64), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_isize(42isize), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_usize(42usize), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_f32(42f32), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_f32(42f32), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_f64(42f64), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_f64(42f64), Some(NotNaN::from(42.0)));
        assert_eq!(NotNaN::<f64>::from_f32(Float::nan()), None);
        assert_eq!(NotNaN::<f64>::from_f64(Float::nan()), None);
    }

    it "should implement ToPrimitive" {
        let x = NotNaN::from(42.0f64);
        assert_eq!(x.to_u8(), Some(42u8));
        assert_eq!(x.to_i8(), Some(42i8));
        assert_eq!(x.to_u16(), Some(42u16));
        assert_eq!(x.to_i16(), Some(42i16));
        assert_eq!(x.to_u32(), Some(42u32));
        assert_eq!(x.to_i32(), Some(42i32));
        assert_eq!(x.to_u64(), Some(42u64));
        assert_eq!(x.to_i64(), Some(42i64));
        assert_eq!(x.to_usize(), Some(42usize));
        assert_eq!(x.to_isize(), Some(42isize));
        assert_eq!(x.to_f32(), Some(42f32));
        assert_eq!(x.to_f32(), Some(42f32));
        assert_eq!(x.to_f64(), Some(42f64));
        assert_eq!(x.to_f64(), Some(42f64));
    }

    it "should implement Num" {
        assert_eq!(NotNaN::<f64>::from_str_radix("42.0", 10).unwrap(), NotNaN::from(42.0f64));
        assert!(NotNaN::<f64>::from_str_radix("NaN", 10).is_err());
    }

    it "should implement Signed" {
        assert_eq!(NotNaN::from(42f64).abs(), NotNaN::from(42f64));
        assert_eq!(NotNaN::from(-42f64).abs(), NotNaN::from(42f64));

        assert_eq!(NotNaN::from(50f64).abs_sub(&NotNaN::from(8f64)), NotNaN::from(42f64));
        assert_eq!(NotNaN::from(8f64).abs_sub(&NotNaN::from(50f64)), NotNaN::from(0f64));
    }
}

describe! hashing {
    it "should hash zero and neg-zero to the same hc" {
        let state = RandomState::new();
        let mut h1 = state.build_hasher();
        let mut h2 = state.build_hasher();
        OrderedFloat::from(0f64).hash(&mut h1);
        OrderedFloat::from(-0f64).hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    it "should hash inf and neg-inf to different hcs" {
        let state = RandomState::new();
        let mut h1 = state.build_hasher();
        let mut h2 = state.build_hasher();
        OrderedFloat::from(f64::INFINITY).hash(&mut h1);
        OrderedFloat::from(f64::NEG_INFINITY).hash(&mut h2);
        assert!(h1.finish() != h2.finish());
    }

    it "should have a good hash function for whole numbers" {
        let state = RandomState::new();
        let limit = 10000;

        let mut set = ::std::collections::HashSet::with_capacity(limit);
        for i in 0..limit {
            let mut h = state.build_hasher();
            OrderedFloat::from(i as f64).hash(&mut h);
            set.insert(h.finish());
        }

        // This allows 100 collisions, which is far too
        // many, but should guard against transient issues
        // that will result from using RandomState
        let pct_unique = set.len() as f64 / limit as f64;
        assert!(0.99f64 < pct_unique, "percent-unique={}", pct_unique);
    }

    it "should have a good hash function for fractional numbers" {
        let state = RandomState::new();
        let limit = 10000;

        let mut set = ::std::collections::HashSet::with_capacity(limit);
        for i in 0..limit {
            let mut h = state.build_hasher();
            OrderedFloat::from(i as f64 * (1f64 / limit as f64)).hash(&mut h);
            set.insert(h.finish());
        }

        // This allows 100 collisions, which is far too
        // many, but should guard against transient issues
        // that will result from using RandomState
        let pct_unique = set.len() as f64 / limit as f64;
        assert!(0.99f64 < pct_unique, "percent-unique={}", pct_unique);
    }
}
