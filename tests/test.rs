extern crate num_traits;
extern crate ordered_float;

pub use ordered_float::*;
pub use num_traits::{Bounded, FromPrimitive, Num, One, Signed, ToPrimitive, Zero};
#[cfg(feature = "std")]
pub use num_traits::Float;
#[cfg(not(feature = "std"))]
pub use num_traits::float::FloatCore as Float;

pub use std::cmp::Ordering::*;
pub use std::convert::TryFrom;
pub use std::{f32, f64, panic};

pub use std::collections::hash_map::RandomState;
pub use std::collections::HashSet;
pub use std::hash::*;

fn not_nan<T: Float>(x: T) -> NotNan<T> {
    NotNan::new(x).unwrap()
}

#[test]
fn ordered_f32_compare_regular_floats() {
    assert_eq!(OrderedFloat(7.0f32).cmp(&OrderedFloat(7.0)), Equal);
    assert_eq!(OrderedFloat(8.0f32).cmp(&OrderedFloat(7.0)), Greater);
    assert_eq!(OrderedFloat(4.0f32).cmp(&OrderedFloat(7.0)), Less);
}

#[test]
fn ordered_f32_compare_regular_floats_op() {
    assert!(OrderedFloat(7.0f32) == OrderedFloat(7.0));
    assert!(OrderedFloat(7.0f32) <= OrderedFloat(7.0));
    assert!(OrderedFloat(7.0f32) >= OrderedFloat(7.0));
    assert!(OrderedFloat(8.0f32) > OrderedFloat(7.0));
    assert!(OrderedFloat(8.0f32) >= OrderedFloat(7.0));
    assert!(OrderedFloat(4.0f32) < OrderedFloat(7.0));
    assert!(OrderedFloat(4.0f32) <= OrderedFloat(7.0));
}

#[test]
fn ordered_f32_compare_nan() {
    let f32_nan: f32 = Float::nan();
    assert_eq!(OrderedFloat(f32_nan).cmp(&OrderedFloat(Float::nan())), Equal);
    assert_eq!(OrderedFloat(f32_nan).cmp(&OrderedFloat(-100000.0f32)), Greater);
    assert_eq!(OrderedFloat(-100.0f32).cmp(&OrderedFloat(Float::nan())), Less);
}

#[test]
fn ordered_f32_compare_nan_op() {
    let f32_nan: OrderedFloat<f32> = OrderedFloat(Float::nan());
    assert!(f32_nan == f32_nan);
    assert!(f32_nan <= f32_nan);
    assert!(f32_nan >= f32_nan);
    assert!(f32_nan > OrderedFloat(-100000.0f32));
    assert!(f32_nan >= OrderedFloat(-100000.0f32));
    assert!(OrderedFloat(-100.0f32) < f32_nan);
    assert!(OrderedFloat(-100.0f32) <= f32_nan);
    assert!(f32_nan > OrderedFloat(Float::infinity()));
    assert!(f32_nan >= OrderedFloat(Float::infinity()));
    assert!(f32_nan > OrderedFloat(Float::neg_infinity()));
    assert!(f32_nan >= OrderedFloat(Float::neg_infinity()));
}

#[test]
fn ordered_f64_compare_regular_floats() {
    assert_eq!(OrderedFloat(7.0f64).cmp(&OrderedFloat(7.0)), Equal);
    assert_eq!(OrderedFloat(8.0f64).cmp(&OrderedFloat(7.0)), Greater);
    assert_eq!(OrderedFloat(4.0f64).cmp(&OrderedFloat(7.0)), Less);
}

#[test]
fn not_nan32_zero() {
    assert_eq!(NotNan::<f32>::zero(), 0.0f32);
    assert!(NotNan::<f32>::zero().is_zero());
}

#[test]
fn not_nan32_one() {
    assert_eq!(NotNan::<f32>::one(), 1.0f32)
}

#[test]
fn not_nan32_bounded() {
    assert_eq!(<NotNan::<f32> as Bounded>::min_value(), <f32 as Bounded>::min_value());
    assert_eq!(<NotNan::<f32> as Bounded>::max_value(), <f32 as Bounded>::max_value());
}

#[test]
fn not_nan32_from_primitive() {
    assert_eq!(NotNan::<f32>::from_i8(42i8), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_u8(42u8), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_i16(42i16), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_u16(42u16), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_i32(42i32), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_u32(42u32), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_i64(42i64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_u64(42u64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_isize(42isize), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_usize(42usize), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_f32(42f32), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_f32(42f32), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_f64(42f64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_f64(42f64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f32>::from_f32(Float::nan()), None);
    assert_eq!(NotNan::<f32>::from_f64(Float::nan()), None);
}

#[test]
fn not_nan32_to_primitive() {
    let x = not_nan(42.0f32);
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

#[test]
fn not_nan32_num() {
    assert_eq!(NotNan::<f32>::from_str_radix("42.0", 10).unwrap(), 42.0f32);
    assert!(NotNan::<f32>::from_str_radix("NaN", 10).is_err());
}

#[test]
fn not_nan32_signed() {
    assert_eq!(not_nan(42f32).abs(), 42f32);
    assert_eq!(not_nan(-42f32).abs(), 42f32);

    assert_eq!(not_nan(50f32).abs_sub(&not_nan(8f32)), 42f32);
    assert_eq!(not_nan(8f32).abs_sub(&not_nan(50f32)), 0f32);
}

#[test]
fn not_nan32_num_cast() {
    assert_eq!(<NotNan<f32> as num_traits::NumCast>::from(42).unwrap(), 42f32);
    assert_eq!(<NotNan<f32> as num_traits::NumCast>::from(f32::nan()), None);
}

#[test]
fn ordered_f64_compare_nan() {
    let f64_nan: f64 = Float::nan();
    assert_eq!(
        OrderedFloat(f64_nan).cmp(&OrderedFloat(Float::nan())),
        Equal
    );
    assert_eq!(
        OrderedFloat(f64_nan).cmp(&OrderedFloat(-100000.0f64)),
        Greater
    );
    assert_eq!(
        OrderedFloat(-100.0f64).cmp(&OrderedFloat(Float::nan())),
        Less
    );
}

#[test]
fn ordered_f64_compare_regular_floats_op() {
    assert!(OrderedFloat(7.0) == OrderedFloat(7.0));
    assert!(OrderedFloat(7.0) <= OrderedFloat(7.0));
    assert!(OrderedFloat(7.0) >= OrderedFloat(7.0));
    assert!(OrderedFloat(8.0) > OrderedFloat(7.0));
    assert!(OrderedFloat(8.0) >= OrderedFloat(7.0));
    assert!(OrderedFloat(4.0) < OrderedFloat(7.0));
    assert!(OrderedFloat(4.0) <= OrderedFloat(7.0));
}

#[test]
fn ordered_f64_compare_nan_op() {
    let f64_nan: OrderedFloat<f64> = OrderedFloat(Float::nan());
    assert!(f64_nan == f64_nan);
    assert!(f64_nan <= f64_nan);
    assert!(f64_nan >= f64_nan);
    assert!(f64_nan > OrderedFloat(-100000.0));
    assert!(f64_nan >= OrderedFloat(-100000.0));
    assert!(OrderedFloat(-100.0) < f64_nan);
    assert!(OrderedFloat(-100.0) <= f64_nan);
    assert!(f64_nan > OrderedFloat(Float::infinity()));
    assert!(f64_nan >= OrderedFloat(Float::infinity()));
    assert!(f64_nan > OrderedFloat(Float::neg_infinity()));
    assert!(f64_nan >= OrderedFloat(Float::neg_infinity()));
}

#[test]
fn not_nan32_compare_regular_floats() {
    assert_eq!(not_nan(7.0f32).cmp(&not_nan(7.0)), Equal);
    assert_eq!(not_nan(8.0f32).cmp(&not_nan(7.0)), Greater);
    assert_eq!(not_nan(4.0f32).cmp(&not_nan(7.0)), Less);
}

#[test]
fn not_nan32_fail_when_constructing_with_nan() {
    let f32_nan: f32 = Float::nan();
    assert!(NotNan::new(f32_nan).is_err());
}

#[test]
fn not_nan32_calculate_correctly() {
    assert_eq!(
        *(not_nan(5.0f32) + not_nan(4.0f32)),
        5.0f32 + 4.0f32
    );
    assert_eq!(*(not_nan(5.0f32) + 4.0f32), 5.0f32 + 4.0f32);
    assert_eq!(
        *(not_nan(5.0f32) - not_nan(4.0f32)),
        5.0f32 - 4.0f32
    );
    assert_eq!(*(not_nan(5.0f32) - 4.0f32), 5.0f32 - 4.0f32);
    assert_eq!(
        *(not_nan(5.0f32) * not_nan(4.0f32)),
        5.0f32 * 4.0f32
    );
    assert_eq!(*(not_nan(5.0f32) * 4.0f32), 5.0f32 * 4.0f32);
    assert_eq!(
        *(not_nan(8.0f32) / not_nan(4.0f32)),
        8.0f32 / 4.0f32
    );
    assert_eq!(*(not_nan(8.0f32) / 4.0f32), 8.0f32 / 4.0f32);
    assert_eq!(
        *(not_nan(8.0f32) % not_nan(4.0f32)),
        8.0f32 % 4.0f32
    );
    assert_eq!(*(not_nan(8.0f32) % 4.0f32), 8.0f32 % 4.0f32);
    assert_eq!(*(-not_nan(1.0f32)), -1.0f32);

    assert!(panic::catch_unwind(|| not_nan(0.0f32) + f32::NAN).is_err());
    assert!(panic::catch_unwind(|| not_nan(0.0f32) - f32::NAN).is_err());
    assert!(panic::catch_unwind(|| not_nan(0.0f32) * f32::NAN).is_err());
    assert!(panic::catch_unwind(|| not_nan(0.0f32) / f32::NAN).is_err());
    assert!(panic::catch_unwind(|| not_nan(0.0f32) % f32::NAN).is_err());

    let mut number = not_nan(5.0f32);
    number += not_nan(4.0f32);
    assert_eq!(*number, 9.0f32);
    number -= not_nan(4.0f32);
    assert_eq!(*number, 5.0f32);
    number *= not_nan(4.0f32);
    assert_eq!(*number, 20.0f32);
    number /= not_nan(4.0f32);
    assert_eq!(*number, 5.0f32);
    number %= not_nan(4.0f32);
    assert_eq!(*number, 1.0f32);

    number = not_nan(5.0f32);
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

    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f32);
            tmp += f32::NAN;
        }).is_err()
    );
    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f32);
            tmp -= f32::NAN;
        }).is_err()
    );
    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f32);
            tmp *= f32::NAN;
        }).is_err()
    );
    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f32);
            tmp /= f32::NAN;
        }).is_err()
    );
    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f32);
            tmp %= f32::NAN;
        }).is_err()
    );
}

#[test]
fn not_nan64_compare_regular_floats() {
    assert_eq!(not_nan(7.0f64).cmp(&not_nan(7.0)), Equal);
    assert_eq!(not_nan(8.0f64).cmp(&not_nan(7.0)), Greater);
    assert_eq!(not_nan(4.0f64).cmp(&not_nan(7.0)), Less);
}

#[test]
fn not_nan64_fail_when_constructing_with_nan() {
    let f64_nan: f64 = Float::nan();
    assert!(NotNan::new(f64_nan).is_err());
}

#[test]
fn not_nan64_calculate_correctly() {
    assert_eq!(
        *(not_nan(5.0f64) + not_nan(4.0f64)),
        5.0f64 + 4.0f64
    );
    assert_eq!(*(not_nan(5.0f64) + 4.0f64), 5.0f64 + 4.0f64);
    assert_eq!(
        *(not_nan(5.0f64) - not_nan(4.0f64)),
        5.0f64 - 4.0f64
    );
    assert_eq!(*(not_nan(5.0f64) - 4.0f64), 5.0f64 - 4.0f64);
    assert_eq!(
        *(not_nan(5.0f64) * not_nan(4.0f64)),
        5.0f64 * 4.0f64
    );
    assert_eq!(*(not_nan(5.0f64) * 4.0f64), 5.0f64 * 4.0f64);
    assert_eq!(
        *(not_nan(8.0f64) / not_nan(4.0f64)),
        8.0f64 / 4.0f64
    );
    assert_eq!(*(not_nan(8.0f64) / 4.0f64), 8.0f64 / 4.0f64);
    assert_eq!(
        *(not_nan(8.0f64) % not_nan(4.0f64)),
        8.0f64 % 4.0f64
    );
    assert_eq!(*(not_nan(8.0f64) % 4.0f64), 8.0f64 % 4.0f64);
    assert_eq!(*(-not_nan(1.0f64)), -1.0f64);

    assert!(panic::catch_unwind(|| not_nan(0.0f64) + f64::NAN).is_err());
    assert!(panic::catch_unwind(|| not_nan(0.0f64) - f64::NAN).is_err());
    assert!(panic::catch_unwind(|| not_nan(0.0f64) * f64::NAN).is_err());
    assert!(panic::catch_unwind(|| not_nan(0.0f64) / f64::NAN).is_err());
    assert!(panic::catch_unwind(|| not_nan(0.0f64) % f64::NAN).is_err());

    let mut number = not_nan(5.0f64);
    number += not_nan(4.0f64);
    assert_eq!(*number, 9.0f64);
    number -= not_nan(4.0f64);
    assert_eq!(*number, 5.0f64);
    number *= not_nan(4.0f64);
    assert_eq!(*number, 20.0f64);
    number /= not_nan(4.0f64);
    assert_eq!(*number, 5.0f64);
    number %= not_nan(4.0f64);
    assert_eq!(*number, 1.0f64);

    number = not_nan(5.0f64);
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

    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f64);
            tmp += f64::NAN;
        }).is_err()
    );
    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f64);
            tmp -= f64::NAN;
        }).is_err()
    );
    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f64);
            tmp *= f64::NAN;
        }).is_err()
    );
    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f64);
            tmp /= f64::NAN;
        }).is_err()
    );
    assert!(
        panic::catch_unwind(|| {
            let mut tmp = not_nan(0.0f64);
            tmp %= f64::NAN;
        }).is_err()
    );
}

#[test]
fn not_nan64_zero() {
    assert_eq!(NotNan::<f64>::zero(), not_nan(0.0f64));
    assert!(NotNan::<f64>::zero().is_zero());
}

#[test]
fn not_nan64_one() {
    assert_eq!(NotNan::<f64>::one(), not_nan(1.0f64))
}

#[test]
fn not_nan64_bounded() {
    assert_eq!(<NotNan::<f64> as Bounded>::min_value(), <f64 as Bounded>::min_value());
    assert_eq!(<NotNan::<f64> as Bounded>::max_value(), <f64 as Bounded>::max_value());
}

#[test]
fn not_nan64_from_primitive() {
    assert_eq!(NotNan::<f64>::from_i8(42i8), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_u8(42u8), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_i16(42i16), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_u16(42u16), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_i32(42i32), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_u32(42u32), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_i64(42i64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_u64(42u64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_isize(42isize), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_usize(42usize), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_f64(42f64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_f64(42f64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_f64(42f64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_f64(42f64), Some(not_nan(42.0)));
    assert_eq!(NotNan::<f64>::from_f64(Float::nan()), None);
    assert_eq!(NotNan::<f64>::from_f64(Float::nan()), None);
}

#[test]
fn not_nan64_to_primitive() {
    let x = not_nan(42.0f64);
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
    assert_eq!(x.to_f64(), Some(42f64));
    assert_eq!(x.to_f64(), Some(42f64));
    assert_eq!(x.to_f64(), Some(42f64));
    assert_eq!(x.to_f64(), Some(42f64));
}

#[test]
fn not_nan64_num() {
    assert_eq!(NotNan::<f64>::from_str_radix("42.0", 10).unwrap(), not_nan(42.0f64));
    assert!(NotNan::<f64>::from_str_radix("NaN", 10).is_err());
}

#[test]
fn not_nan64_signed() {
    assert_eq!(not_nan(42f64).abs(), not_nan(42f64));
    assert_eq!(not_nan(-42f64).abs(), not_nan(42f64));

    assert_eq!(not_nan(50f64).abs_sub(&not_nan(8f64)), not_nan(42f64));
    assert_eq!(not_nan(8f64).abs_sub(&not_nan(50f64)), not_nan(0f64));
}

#[test]
fn not_nan64_num_cast() {
    assert_eq!(<NotNan<f64> as num_traits::NumCast>::from(42), Some(not_nan(42f64)));
    assert_eq!(<NotNan<f64> as num_traits::NumCast>::from(f64::nan()), None);
}

#[test]
fn hash_zero_and_neg_zero_to_the_same_hc() {
    let state = RandomState::new();
    let mut h1 = state.build_hasher();
    let mut h2 = state.build_hasher();
    OrderedFloat::from(0f64).hash(&mut h1);
    OrderedFloat::from(-0f64).hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn hash_inf_and_neg_inf_to_different_hcs() {
    let state = RandomState::new();
    let mut h1 = state.build_hasher();
    let mut h2 = state.build_hasher();
    OrderedFloat::from(f64::INFINITY).hash(&mut h1);
    OrderedFloat::from(f64::NEG_INFINITY).hash(&mut h2);
    assert!(h1.finish() != h2.finish());
}

#[test]
fn hash_is_good_for_whole_numbers() {
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

#[test]
fn hash_is_good_for_fractional_numbers() {
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

#[test]
#[should_panic]
fn test_add_fails_on_nan() {
    let a = not_nan(std::f32::INFINITY);
    let b = not_nan(std::f32::NEG_INFINITY);
    let _c = a + b;
}

#[test]
fn ordered_f32_neg() {
    assert_eq!(OrderedFloat(-7.0f32), -OrderedFloat(7.0f32));
}

#[test]
fn ordered_f64_neg() {
    assert_eq!(OrderedFloat(-7.0f64), -OrderedFloat(7.0f64));
}

#[test]
#[should_panic]
fn test_sum_fails_on_nan() {
    let a = not_nan(std::f32::INFINITY);
    let b = not_nan(std::f32::NEG_INFINITY);
    let _c: NotNan<_> = [a,b].iter().sum();
}

#[test]
#[should_panic]
fn test_product_fails_on_nan() {
    let a = not_nan(std::f32::INFINITY);
    let b = not_nan(0f32);
    let _c: NotNan<_> = [a,b].iter().product();
}

#[test]
fn not_nan64_sum_product() {
    let a = not_nan(2138.1237);
    let b = not_nan(132f64);
    let c = not_nan(5.1);

    assert_eq!(std::iter::empty::<NotNan<f64>>().sum::<NotNan<_>>(), NotNan::new(0f64).unwrap());
    assert_eq!([a].iter().sum::<NotNan<_>>(), a);
    assert_eq!([a,b].iter().sum::<NotNan<_>>(), a + b);
    assert_eq!([a,b,c].iter().sum::<NotNan<_>>(), a + b + c);

    assert_eq!(std::iter::empty::<NotNan<f64>>().product::<NotNan<_>>(), NotNan::new(1f64).unwrap());
    assert_eq!([a].iter().product::<NotNan<_>>(), a);
    assert_eq!([a,b].iter().product::<NotNan<_>>(), a * b);
    assert_eq!([a,b,c].iter().product::<NotNan<_>>(), a * b * c);

}

#[test]
fn not_nan_usage_in_const_context() {
    const A: NotNan<f32> = unsafe { NotNan::unchecked_new(111f32) };
    assert_eq!(A, NotNan::new(111f32).unwrap());
}

#[test]
fn not_nan_panic_safety() {
    let catch_op = |mut num, op: fn(&mut NotNan<_>)| {
        let mut num_ref = panic::AssertUnwindSafe(&mut num);
        let _ = panic::catch_unwind(move || op(&mut *num_ref));
        num
    };

    assert!(!catch_op(not_nan(f32::INFINITY), |a| *a += f32::NEG_INFINITY).is_nan());
    assert!(!catch_op(not_nan(f32::INFINITY), |a| *a -= f32::INFINITY).is_nan());
    assert!(!catch_op(not_nan(0.0), |a| *a *= f32::INFINITY).is_nan());
    assert!(!catch_op(not_nan(0.0), |a| *a /= 0.0).is_nan());
    assert!(!catch_op(not_nan(0.0), |a| *a %= 0.0).is_nan());
}

#[test]
fn from_ref() {
    let f = 1.0f32;
    let o: &OrderedFloat<f32> = (&f).into();
    assert_eq!(*o, 1.0f32);

    let mut f = 1.0f64;
    let o: &OrderedFloat<f64> = (&f).into();
    assert_eq!(*o, 1.0f64);

    let o: &mut OrderedFloat<f64> = (&mut f).into();
    assert_eq!(*o, 1.0f64);
    *o = OrderedFloat(2.0);
    assert_eq!(*o, 2.0f64);
    assert_eq!(f, 2.0f64);
}
