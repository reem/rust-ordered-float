#![no_std]
#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]

//! Wrappers for total order on Floats.

extern crate num_traits;
#[cfg(feature = "std")] extern crate std;
#[cfg(feature = "std")] use std::error::Error;

use core::cmp::Ordering;
use core::convert::TryFrom;
use core::num::FpCategory;
use core::ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Rem,
               RemAssign, Sub, SubAssign};
use core::hash::{Hash, Hasher};
use core::fmt;
use core::mem;
use core::hint::unreachable_unchecked;
use core::iter::{Sum, Product};
use core::str::FromStr;

use num_traits::{Bounded, FromPrimitive, Num, NumCast, One, Signed, ToPrimitive, Zero};
#[cfg(feature = "std")]
use num_traits::Float;
#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore as Float;

// masks for the parts of the IEEE 754 float
const SIGN_MASK: u64 = 0x8000000000000000u64;
const EXP_MASK: u64 = 0x7ff0000000000000u64;
const MAN_MASK: u64 = 0x000fffffffffffffu64;

// canonical raw bit patterns (for hashing)
const CANONICAL_NAN_BITS: u64 = 0x7ff8000000000000u64;
const CANONICAL_ZERO_BITS: u64 = 0x0u64;

/// A wrapper around Floats providing an implementation of Ord and Hash.
///
/// NaN is sorted as *greater* than all other values and *equal*
/// to itself, in contradiction with the IEEE standard.
#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct OrderedFloat<T>(pub T);

impl<T: Float> OrderedFloat<T> {
    /// Get the value out.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Float> AsRef<T> for OrderedFloat<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Float> AsMut<T> for OrderedFloat<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<'a, T: Float> From<&'a T> for &'a OrderedFloat<T> {
    fn from(t: &'a T) -> &'a OrderedFloat<T> {
        // Safety: OrderedFloat is #[repr(transparent)] and has no invalid values.
        unsafe {
            &*(t as *const T as *const OrderedFloat<T>)
        }
    }
}

impl<'a, T: Float> From<&'a mut T> for &'a mut OrderedFloat<T> {
    fn from(t: &'a mut T) -> &'a mut OrderedFloat<T> {
        // Safety: OrderedFloat is #[repr(transparent)] and has no invalid values.
        unsafe {
            &mut *(t as *mut T as *mut OrderedFloat<T>)
        }
    }
}

impl<T: Float> PartialOrd for OrderedFloat<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Float> Ord for OrderedFloat<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = self.as_ref();
        let rhs = other.as_ref();
        match lhs.partial_cmp(&rhs) {
            Some(ordering) => ordering,
            None => {
                if lhs.is_nan() {
                    if rhs.is_nan() {
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

impl<T: Float> PartialEq for OrderedFloat<T> {
    fn eq(&self, other: &OrderedFloat<T>) -> bool {
        if self.as_ref().is_nan() {
            other.as_ref().is_nan()
        } else {
            self.as_ref() == other.as_ref()
        }
    }
}

impl<T: Float> PartialEq<T> for OrderedFloat<T> {
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<T: Float> Hash for OrderedFloat<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.is_nan() {
            // normalize to one representation of NaN
            hash_float(&T::nan(), state)
        } else {
            hash_float(self.as_ref(), state)
        }
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

impl<T: Float> Eq for OrderedFloat<T> {}

impl<T: Add> Add for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn add(self, other: Self) -> Self::Output {
        OrderedFloat(self.0 + other.0)
    }
}

impl<T: Add> Add<T> for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn add(self, other: T) -> Self::Output {
        OrderedFloat(self.0 + other)
    }
}

impl<T: Sub> Sub for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn sub(self, other: Self) -> Self::Output {
        OrderedFloat(self.0 - other.0)
    }
}

impl<T: Sub> Sub<T> for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn sub(self, other: T) -> Self::Output {
        OrderedFloat(self.0 - other)
    }
}

impl<T: Mul> Mul for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn mul(self, other: Self) -> Self::Output {
        OrderedFloat(self.0 * other.0)
    }
}

impl<T: Mul> Mul<T> for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn mul(self, other: T) -> Self::Output {
        OrderedFloat(self.0 * other)
    }
}

impl<T: Div> Div for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn div(self, other: Self) -> Self::Output {
        OrderedFloat(self.0 / other.0)
    }
}

impl<T: Div> Div<T> for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn div(self, other: T) -> Self::Output {
        OrderedFloat(self.0 / other)
    }
}

impl<T: Rem> Rem for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn rem(self, other: Self) -> Self::Output {
        OrderedFloat(self.0 % other.0)
    }
}

impl<T: Rem> Rem<T> for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn rem(self, other: T) -> Self::Output {
        OrderedFloat(self.0 % other)
    }
}

impl<T: Bounded> Bounded for OrderedFloat<T> {
    fn min_value() -> Self {
        OrderedFloat(T::min_value())
    }

    fn max_value() -> Self {
        OrderedFloat(T::max_value())
    }
}

impl<T: FromStr> FromStr for OrderedFloat<T> {
    type Err = T::Err;

    /// Convert a &str to `OrderedFloat`. Returns an error if the string fails to parse.
    ///
    /// ```
    /// use ordered_float::OrderedFloat;
    ///
    /// assert!("-10".parse::<OrderedFloat<f32>>().is_ok());
    /// assert!("abc".parse::<OrderedFloat<f32>>().is_err());
    /// assert!("NaN".parse::<OrderedFloat<f32>>().is_ok());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(OrderedFloat)
    }
}

impl<T: Neg> Neg for OrderedFloat<T> {
    type Output = OrderedFloat<T::Output>;

    fn neg(self) -> Self::Output {
        OrderedFloat(-self.0)
    }
}

impl<T: Zero> Zero for OrderedFloat<T> {
    fn zero() -> Self { OrderedFloat(T::zero()) }

    fn is_zero(&self) -> bool { self.0.is_zero() }
}

impl<T: One> One for OrderedFloat<T> {
    fn one() -> Self { OrderedFloat(T::one()) }
}

impl<T: NumCast> NumCast for OrderedFloat<T> {
    fn from<F: ToPrimitive>(n: F) -> Option<Self> {
        T::from(n).map(OrderedFloat)
    }
}

impl<T: FromPrimitive> FromPrimitive for OrderedFloat<T> {
    fn from_i64(n: i64) -> Option<Self> { T::from_i64(n).map(OrderedFloat) }
    fn from_u64(n: u64) -> Option<Self> { T::from_u64(n).map(OrderedFloat) }
    fn from_isize(n: isize) -> Option<Self> { T::from_isize(n).map(OrderedFloat) }
    fn from_i8(n: i8) -> Option<Self> { T::from_i8(n).map(OrderedFloat) }
    fn from_i16(n: i16) -> Option<Self> { T::from_i16(n).map(OrderedFloat) }
    fn from_i32(n: i32) -> Option<Self> { T::from_i32(n).map(OrderedFloat) }
    fn from_usize(n: usize) -> Option<Self> { T::from_usize(n).map(OrderedFloat) }
    fn from_u8(n: u8) -> Option<Self> { T::from_u8(n).map(OrderedFloat) }
    fn from_u16(n: u16) -> Option<Self> { T::from_u16(n).map(OrderedFloat) }
    fn from_u32(n: u32) -> Option<Self> { T::from_u32(n).map(OrderedFloat) }
    fn from_f32(n: f32) -> Option<Self> { T::from_f32(n).map(OrderedFloat) }
    fn from_f64(n: f64) -> Option<Self> { T::from_f64(n).map(OrderedFloat) }
}

impl<T: ToPrimitive> ToPrimitive for OrderedFloat<T> {
    fn to_i64(&self) -> Option<i64> { self.0.to_i64() }
    fn to_u64(&self) -> Option<u64> { self.0.to_u64() }
    fn to_isize(&self) -> Option<isize> { self.0.to_isize() }
    fn to_i8(&self) -> Option<i8> { self.0.to_i8() }
    fn to_i16(&self) -> Option<i16> { self.0.to_i16() }
    fn to_i32(&self) -> Option<i32> { self.0.to_i32() }
    fn to_usize(&self) -> Option<usize> { self.0.to_usize() }
    fn to_u8(&self) -> Option<u8> { self.0.to_u8() }
    fn to_u16(&self) -> Option<u16> { self.0.to_u16() }
    fn to_u32(&self) -> Option<u32> { self.0.to_u32() }
    fn to_f32(&self) -> Option<f32> { self.0.to_f32() }
    fn to_f64(&self) -> Option<f64> { self.0.to_f64() }
}

impl<T: Float> num_traits::float::FloatCore for OrderedFloat<T> {
    fn nan() -> Self { OrderedFloat(T::nan()) }
    fn infinity() -> Self { OrderedFloat(T::infinity()) }
    fn neg_infinity() -> Self { OrderedFloat(T::neg_infinity()) }
    fn neg_zero() -> Self { OrderedFloat(T::neg_zero()) }
    fn min_value() -> Self { OrderedFloat(T::min_value()) }
    fn min_positive_value() -> Self { OrderedFloat(T::min_positive_value()) }
    fn max_value() -> Self { OrderedFloat(T::max_value()) }
    fn is_nan(self) -> bool { self.0.is_nan() }
    fn is_infinite(self) -> bool { self.0.is_infinite() }
    fn is_finite(self) -> bool { self.0.is_finite() }
    fn is_normal(self) -> bool { self.0.is_normal() }
    fn classify(self) -> FpCategory { self.0.classify() }
    fn floor(self) -> Self { OrderedFloat(self.0.floor()) }
    fn ceil(self) -> Self { OrderedFloat(self.0.ceil()) }
    fn round(self) -> Self { OrderedFloat(self.0.round()) }
    fn trunc(self) -> Self { OrderedFloat(self.0.trunc()) }
    fn fract(self) -> Self { OrderedFloat(self.0.fract()) }
    fn abs(self) -> Self { OrderedFloat(self.0.abs()) }
    fn signum(self) -> Self { OrderedFloat(self.0.signum()) }
    fn is_sign_positive(self) -> bool { self.0.is_sign_positive() }
    fn is_sign_negative(self) -> bool { self.0.is_sign_negative() }
    fn recip(self) -> Self { OrderedFloat(self.0.recip()) }
    fn powi(self, n: i32) -> Self { OrderedFloat(self.0.powi(n)) }
    fn integer_decode(self) -> (u64, i16, i8) { self.0.integer_decode() }
    fn epsilon() -> Self { OrderedFloat(T::epsilon()) }
    fn to_degrees(self) -> Self { OrderedFloat(self.0.to_degrees()) }
    fn to_radians(self) -> Self { OrderedFloat(self.0.to_radians()) }
}

#[cfg(feature = "std")]
impl<T: Float> Float for OrderedFloat<T> {
    fn nan() -> Self { OrderedFloat(T::nan()) }
    fn infinity() -> Self { OrderedFloat(T::infinity()) }
    fn neg_infinity() -> Self { OrderedFloat(T::neg_infinity()) }
    fn neg_zero() -> Self { OrderedFloat(T::neg_zero()) }
    fn min_value() -> Self { OrderedFloat(T::min_value()) }
    fn min_positive_value() -> Self { OrderedFloat(T::min_positive_value()) }
    fn max_value() -> Self { OrderedFloat(T::max_value()) }
    fn is_nan(self) -> bool { self.0.is_nan() }
    fn is_infinite(self) -> bool { self.0.is_infinite() }
    fn is_finite(self) -> bool { self.0.is_finite() }
    fn is_normal(self) -> bool { self.0.is_normal() }
    fn classify(self) -> FpCategory { self.0.classify() }
    fn floor(self) -> Self { OrderedFloat(self.0.floor()) }
    fn ceil(self) -> Self { OrderedFloat(self.0.ceil()) }
    fn round(self) -> Self { OrderedFloat(self.0.round()) }
    fn trunc(self) -> Self { OrderedFloat(self.0.trunc()) }
    fn fract(self) -> Self { OrderedFloat(self.0.fract()) }
    fn abs(self) -> Self { OrderedFloat(self.0.abs()) }
    fn signum(self) -> Self { OrderedFloat(self.0.signum()) }
    fn is_sign_positive(self) -> bool { self.0.is_sign_positive() }
    fn is_sign_negative(self) -> bool { self.0.is_sign_negative() }
    fn mul_add(self, a: Self, b: Self) -> Self { OrderedFloat(self.0.mul_add(a.0, b.0)) }
    fn recip(self) -> Self { OrderedFloat(self.0.recip()) }
    fn powi(self, n: i32) -> Self { OrderedFloat(self.0.powi(n)) }
    fn powf(self, n: Self) -> Self { OrderedFloat(self.0.powf(n.0)) }
    fn sqrt(self) -> Self { OrderedFloat(self.0.sqrt()) }
    fn exp(self) -> Self { OrderedFloat(self.0.exp()) }
    fn exp2(self) -> Self { OrderedFloat(self.0.exp2()) }
    fn ln(self) -> Self { OrderedFloat(self.0.ln()) }
    fn log(self, base: Self) -> Self { OrderedFloat(self.0.log(base.0)) }
    fn log2(self) -> Self { OrderedFloat(self.0.log2()) }
    fn log10(self) -> Self { OrderedFloat(self.0.log10()) }
    fn max(self, other: Self) -> Self { OrderedFloat(self.0.max(other.0)) }
    fn min(self, other: Self) -> Self { OrderedFloat(self.0.min(other.0)) }
    fn abs_sub(self, other: Self) -> Self { OrderedFloat(self.0.abs_sub(other.0)) }
    fn cbrt(self) -> Self { OrderedFloat(self.0.cbrt()) }
    fn hypot(self, other: Self) -> Self { OrderedFloat(self.0.hypot(other.0)) }
    fn sin(self) -> Self { OrderedFloat(self.0.sin()) }
    fn cos(self) -> Self { OrderedFloat(self.0.cos()) }
    fn tan(self) -> Self { OrderedFloat(self.0.tan()) }
    fn asin(self) -> Self { OrderedFloat(self.0.asin()) }
    fn acos(self) -> Self { OrderedFloat(self.0.acos()) }
    fn atan(self) -> Self { OrderedFloat(self.0.atan()) }
    fn atan2(self, other: Self) -> Self { OrderedFloat(self.0.atan2(other.0)) }
    fn sin_cos(self) -> (Self, Self) {
        let (a, b) = self.0.sin_cos();
        (OrderedFloat(a), OrderedFloat(b))
    }
    fn exp_m1(self) -> Self { OrderedFloat(self.0.exp_m1()) }
    fn ln_1p(self) -> Self { OrderedFloat(self.0.ln_1p()) }
    fn sinh(self) -> Self { OrderedFloat(self.0.sinh()) }
    fn cosh(self) -> Self { OrderedFloat(self.0.cosh()) }
    fn tanh(self) -> Self { OrderedFloat(self.0.tanh()) }
    fn asinh(self) -> Self { OrderedFloat(self.0.asinh()) }
    fn acosh(self) -> Self { OrderedFloat(self.0.acosh()) }
    fn atanh(self) -> Self { OrderedFloat(self.0.atanh()) }
    fn integer_decode(self) -> (u64, i16, i8) { self.0.integer_decode() }
    fn epsilon() -> Self { OrderedFloat(T::epsilon()) }
    fn to_degrees(self) -> Self { OrderedFloat(self.0.to_degrees()) }
    fn to_radians(self) -> Self { OrderedFloat(self.0.to_radians()) }
}


impl<T: Float + Num> Num for OrderedFloat<T> {
    type FromStrRadixErr = T::FromStrRadixErr;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        T::from_str_radix(str, radix).map(OrderedFloat)
    }
}

/// A wrapper around Floats providing an implementation of Ord and Hash.
///
/// A NaN value cannot be stored in this type.
#[derive(PartialOrd, PartialEq, Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct NotNan<T>(T);

impl<T> NotNan<T> {
    /// Create a NotNan value from a value that is guaranteed to not be NaN
    ///
    /// # Safety
    ///
    /// Behaviour is undefined if `val` is NaN
    pub const unsafe fn unchecked_new(val: T) -> Self {
        NotNan(val)
    }
}

impl<T: Float> NotNan<T> {
    /// Create a NotNan value.
    ///
    /// Returns Err if val is NaN
    pub fn new(val: T) -> Result<Self, FloatIsNan> {
        match val {
            ref val if val.is_nan() => Err(FloatIsNan),
            val => Ok(NotNan(val)),
        }
    }

    /// Get the value out.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Float> AsRef<T> for NotNan<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Float> Ord for NotNan<T> {
    fn cmp(&self, other: &NotNan<T>) -> Ordering {
        match self.partial_cmp(&other) {
            Some(ord) => ord,
            None => unsafe { unreachable_unchecked() },
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl<T: Float> Hash for NotNan<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_float(self.as_ref(), state)
    }
}

impl<T: Float + fmt::Display> fmt::Display for NotNan<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl From<NotNan<f32>> for f32 {
    fn from(value: NotNan<f32>) -> Self {
        value.into_inner()
    }
}

impl From<NotNan<f64>> for f64 {
    fn from(value: NotNan<f64>) -> Self {
        value.into_inner()
    }
}

impl TryFrom<f32> for NotNan<f32> {
    type Error = FloatIsNan;
    fn try_from(v: f32) -> Result<Self, Self::Error> {
        NotNan::new(v)
    }
}

impl TryFrom<f64> for NotNan<f64> {
    type Error = FloatIsNan;
    fn try_from(v: f64) -> Result<Self, Self::Error> {
        NotNan::new(v)
    }
}

impl From<NotNan<f32>> for NotNan<f64> {
    fn from(v: NotNan<f32>) -> NotNan<f64> {
        unsafe {
            NotNan::unchecked_new(v.0 as f64)
        }
    }
}

impl<T: Float> Deref for NotNan<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: Float + PartialEq> Eq for NotNan<T> {}

impl<T: Float> PartialEq<T> for NotNan<T> {
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

/// Adds two NotNans.
///
/// Panics if the computation results in NaN
impl<T: Float> Add for NotNan<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self + other.0
    }
}

/// Adds a float directly.
///
/// Panics if the provided value is NaN or the computation results in NaN
impl<T: Float> Add<T> for NotNan<T> {
    type Output = Self;

    fn add(self, other: T) -> Self {
        NotNan::new(self.0 + other).expect("Addition resulted in NaN")
    }
}

impl<T: Float + AddAssign> AddAssign for NotNan<T> {
    fn add_assign(&mut self, other: Self) {
        *self += other.0;
    }
}

/// Adds a float directly.
///
/// Panics if the provided value is NaN.
impl<T: Float + AddAssign> AddAssign<T> for NotNan<T> {
    fn add_assign(&mut self, other: T) {
        *self = *self + other;
    }
}


impl<T: Float + Sum> Sum for NotNan<T> {
    fn sum<I: Iterator<Item = NotNan<T>>>(iter: I) -> Self {
        NotNan::new(iter.map(|v| v.0).sum()).expect("Sum resulted in NaN")
    }
}

impl<'a, T: Float + Sum + 'a> Sum<&'a NotNan<T>> for NotNan<T> {
    fn sum<I: Iterator<Item = &'a NotNan<T>>>(iter: I) -> Self {
        iter.cloned().sum()
    }
}

impl<T: Float> Sub for NotNan<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self - other.0
    }
}

/// Subtracts a float directly.
///
/// Panics if the provided value is NaN or the computation results in NaN
impl<T: Float> Sub<T> for NotNan<T> {
    type Output = Self;

    fn sub(self, other: T) -> Self {
        NotNan::new(self.0 - other).expect("Subtraction resulted in NaN")
    }
}

impl<T: Float + SubAssign> SubAssign for NotNan<T> {
    fn sub_assign(&mut self, other: Self) {
        *self -= other.0
    }
}

/// Subtracts a float directly.
///
/// Panics if the provided value is NaN or the computation results in NaN
impl<T: Float + SubAssign> SubAssign<T> for NotNan<T> {
    fn sub_assign(&mut self, other: T) {
        *self = *self - other;
    }
}

impl<T: Float> Mul for NotNan<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self * other.0
    }
}

/// Multiplies a float directly.
///
/// Panics if the provided value is NaN or the computation results in NaN
impl<T: Float> Mul<T> for NotNan<T> {
    type Output = Self;

    fn mul(self, other: T) -> Self {
        NotNan::new(self.0 * other).expect("Multiplication resulted in NaN")
    }
}

impl<T: Float + MulAssign> MulAssign for NotNan<T> {
    fn mul_assign(&mut self, other: Self) {
        *self *= other.0
    }
}

/// Multiplies a float directly.
///
/// Panics if the provided value is NaN.
impl<T: Float + MulAssign> MulAssign<T> for NotNan<T> {
    fn mul_assign(&mut self, other: T) {
        *self = *self * other;
    }
}

impl<T: Float + Product> Product for NotNan<T> {
    fn product<I: Iterator<Item = NotNan<T>>>(iter: I) -> Self {
        NotNan::new(iter.map(|v| v.0).product()).expect("Product resulted in NaN")
    }
}

impl<'a, T: Float + Product + 'a> Product<&'a NotNan<T>> for NotNan<T> {
    fn product<I: Iterator<Item = &'a NotNan<T>>>(iter: I) -> Self {
        iter.cloned().product()
    }
}

impl<T: Float> Div for NotNan<T> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        self / other.0
    }
}

/// Divides a float directly.
///
/// Panics if the provided value is NaN or the computation results in NaN
impl<T: Float> Div<T> for NotNan<T> {
    type Output = Self;

    fn div(self, other: T) -> Self {
        NotNan::new(self.0 / other).expect("Division resulted in NaN")
    }
}

impl<T: Float + DivAssign> DivAssign for NotNan<T> {
    fn div_assign(&mut self, other: Self) {
        *self /= other.0;
    }
}

/// Divides a float directly.
///
/// Panics if the provided value is NaN or the computation results in NaN
impl<T: Float + DivAssign> DivAssign<T> for NotNan<T> {
    fn div_assign(&mut self, other: T) {
        *self = *self / other;
    }
}

impl<T: Float> Rem for NotNan<T> {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        self % other.0
    }
}

/// Calculates `%` with a float directly.
///
/// Panics if the provided value is NaN or the computation results in NaN
impl<T: Float> Rem<T> for NotNan<T> {
    type Output = Self;

    fn rem(self, other: T) -> Self {
        NotNan::new(self.0 % other).expect("Rem resulted in NaN")
    }
}

impl<T: Float + RemAssign> RemAssign for NotNan<T> {
    fn rem_assign(&mut self, other: Self) {
        *self %= other.0
    }
}

/// Calculates `%=` with a float directly.
///
/// Panics if the provided value is NaN or the computation results in NaN
impl<T: Float + RemAssign> RemAssign<T> for NotNan<T> {
    fn rem_assign(&mut self, other: T) {
        *self = *self % other;
    }
}

impl<T: Float> Neg for NotNan<T> {
    type Output = Self;

    fn neg(self) -> Self {
        NotNan(-self.0)
    }
}

/// An error indicating an attempt to construct NotNan from a NaN
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FloatIsNan;

#[cfg(feature = "std")]
impl Error for FloatIsNan {
    fn description(&self) -> &str {
        "NotNan constructed with NaN"
    }
}

impl fmt::Display for FloatIsNan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NotNan constructed with NaN")
    }
}

#[cfg(feature = "std")]
impl Into<std::io::Error> for FloatIsNan {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, self)
    }
}

#[inline]
fn hash_float<F: Float, H: Hasher>(f: &F, state: &mut H) {
    raw_double_bits(f).hash(state);
}

#[inline]
fn raw_double_bits<F: Float>(f: &F) -> u64 {
    if f.is_nan() {
        return CANONICAL_NAN_BITS;
    }

    let (man, exp, sign) = f.integer_decode();
    if man == 0 {
        return CANONICAL_ZERO_BITS;
    }

    let exp_u64 = unsafe { mem::transmute::<i16, u16>(exp) } as u64;
    let sign_u64 = if sign > 0 { 1u64 } else { 0u64 };
    (man & MAN_MASK) | ((exp_u64 << 52) & EXP_MASK) | ((sign_u64 << 63) & SIGN_MASK)
}

impl<T: Float> Zero for NotNan<T> {
    fn zero() -> Self { NotNan(T::zero()) }

    fn is_zero(&self) -> bool { self.0.is_zero() }
}

impl<T: Float> One for NotNan<T> {
    fn one() -> Self { NotNan(T::one()) }
}

impl<T: Float> Bounded for NotNan<T> {
    fn min_value() -> Self {
        NotNan(T::min_value())
    }

    fn max_value() -> Self {
        NotNan(T::max_value())
    }
}

impl<T: Float + FromStr> FromStr for NotNan<T> {
    type Err = ParseNotNanError<T::Err>;

    /// Convert a &str to `NotNan`. Returns an error if the string fails to parse,
    /// or if the resulting value is NaN
    ///
    /// ```
    /// use ordered_float::NotNan;
    ///
    /// assert!("-10".parse::<NotNan<f32>>().is_ok());
    /// assert!("abc".parse::<NotNan<f32>>().is_err());
    /// assert!("NaN".parse::<NotNan<f32>>().is_err());
    /// ```
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        src.parse()
            .map_err(ParseNotNanError::ParseFloatError)
            .and_then(|f| NotNan::new(f).map_err(|_| ParseNotNanError::IsNaN))
    }
}

impl<T: Float + FromPrimitive> FromPrimitive for NotNan<T> {
    fn from_i64(n: i64) -> Option<Self> { T::from_i64(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_u64(n: u64) -> Option<Self> { T::from_u64(n).and_then(|n| NotNan::new(n).ok()) }

    fn from_isize(n: isize) -> Option<Self> { T::from_isize(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_i8(n: i8) -> Option<Self> { T::from_i8(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_i16(n: i16) -> Option<Self> { T::from_i16(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_i32(n: i32) -> Option<Self> { T::from_i32(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_usize(n: usize) -> Option<Self> { T::from_usize(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_u8(n: u8) -> Option<Self> { T::from_u8(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_u16(n: u16) -> Option<Self> { T::from_u16(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_u32(n: u32) -> Option<Self> { T::from_u32(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_f32(n: f32) -> Option<Self> { T::from_f32(n).and_then(|n| NotNan::new(n).ok()) }
    fn from_f64(n: f64) -> Option<Self> { T::from_f64(n).and_then(|n| NotNan::new(n).ok()) }
}

impl<T: Float> ToPrimitive for NotNan<T> {
    fn to_i64(&self) -> Option<i64> { self.0.to_i64() }
    fn to_u64(&self) -> Option<u64> { self.0.to_u64() }

    fn to_isize(&self) -> Option<isize> { self.0.to_isize() }
    fn to_i8(&self) -> Option<i8> { self.0.to_i8() }
    fn to_i16(&self) -> Option<i16> { self.0.to_i16() }
    fn to_i32(&self) -> Option<i32> { self.0.to_i32() }
    fn to_usize(&self) -> Option<usize> { self.0.to_usize() }
    fn to_u8(&self) -> Option<u8> { self.0.to_u8() }
    fn to_u16(&self) -> Option<u16> { self.0.to_u16() }
    fn to_u32(&self) -> Option<u32> { self.0.to_u32() }
    fn to_f32(&self) -> Option<f32> { self.0.to_f32() }
    fn to_f64(&self) -> Option<f64> { self.0.to_f64() }
}

/// An error indicating a parse error from a string for `NotNan`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ParseNotNanError<E> {
    /// A plain parse error from the underlying float type.
    ParseFloatError(E),
    /// The parsed float value resulted in a NaN.
    IsNaN,
}

#[cfg(feature = "std")]
impl<E: fmt::Debug + Error + 'static> Error for ParseNotNanError<E> {
    fn description(&self) -> &str {
        "Error parsing a not-NaN floating point value"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseNotNanError::ParseFloatError(e) => Some(e),
            ParseNotNanError::IsNaN => None,
        }
    }
}

impl<E: fmt::Display> fmt::Display for ParseNotNanError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseNotNanError::ParseFloatError(e) => write!(f, "Parse error: {}", e),
            ParseNotNanError::IsNaN => write!(f, "NotNan parser encounter a NaN"),
        }
    }
}

impl<T: Float> Num for NotNan<T> {
    type FromStrRadixErr = ParseNotNanError<T::FromStrRadixErr>;

    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        T::from_str_radix(src, radix)
            .map_err(ParseNotNanError::ParseFloatError)
            .and_then(|n| NotNan::new(n).map_err(|_| ParseNotNanError::IsNaN))
    }
}

impl<T: Float + Signed> Signed for NotNan<T> {
    fn abs(&self) -> Self { NotNan(self.0.abs()) }

    fn abs_sub(&self, other: &Self) -> Self {
        NotNan::new(Signed::abs_sub(&self.0, &other.0)).expect("Subtraction resulted in NaN")
    }

    fn signum(&self) -> Self { NotNan(self.0.signum()) }
    fn is_positive(&self) -> bool { self.0.is_positive() }
    fn is_negative(&self) -> bool { self.0.is_negative() }
}

impl<T: Float> NumCast for NotNan<T> {
    fn from<F: ToPrimitive>(n: F) -> Option<Self> {
        T::from(n).and_then(|n| NotNan::new(n).ok())
    }
}

#[cfg(feature = "serde")]
mod impl_serde {
    extern crate serde;
    use self::serde::{Serialize, Serializer, Deserialize, Deserializer};
    use self::serde::de::{Error, Unexpected};
    use super::{OrderedFloat, NotNan};
    #[cfg(feature = "std")]
    use num_traits::Float;
    #[cfg(not(feature = "std"))]
    use num_traits::float::FloatCore as Float;
    use core::f64;

    #[cfg(test)]
    extern crate serde_test;
    #[cfg(test)]
    use self::serde_test::{Token, assert_tokens, assert_de_tokens_error};

    impl<T: Float + Serialize> Serialize for OrderedFloat<T> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            self.0.serialize(s)
        }
    }

    impl<'de, T: Float + Deserialize<'de>> Deserialize<'de> for OrderedFloat<T> {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            T::deserialize(d).map(OrderedFloat)
        }
    }

    impl<T: Float + Serialize> Serialize for NotNan<T> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            self.0.serialize(s)
        }
    }

    impl<'de, T: Float + Deserialize<'de>> Deserialize<'de> for NotNan<T> {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let float = T::deserialize(d)?;
            NotNan::new(float).map_err(|_| {
                Error::invalid_value(Unexpected::Float(f64::NAN), &"float (but not NaN)")
            })
        }
    }

    #[test]
    fn test_ordered_float() {
        let float = OrderedFloat(1.0f64);
        assert_tokens(&float, &[Token::F64(1.0)]);
    }

    #[test]
    fn test_not_nan() {
        let float = NotNan(1.0f64);
        assert_tokens(&float, &[Token::F64(1.0)]);
    }

    #[test]
    fn test_fail_on_nan() {
        assert_de_tokens_error::<NotNan<f64>>(
            &[Token::F64(f64::NAN)],
            "invalid value: floating point `NaN`, expected float (but not NaN)");
    }
}
