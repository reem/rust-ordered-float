#![no_std]
#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
#![allow(clippy::derive_partial_eq_without_eq)]

//! Wrappers for total order on Floats.  See the [`OrderedFloat`] and [`NotNan`] docs for details.

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
use std::error::Error;

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter::{Product, Sum};
use core::num::FpCategory;
use core::ops::{
    Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
    SubAssign,
};
use core::str::FromStr;

pub use num_traits::float::FloatCore;
#[cfg(any(feature = "std", feature = "libm"))]
use num_traits::real::Real;
use num_traits::{
    AsPrimitive, Bounded, FloatConst, FromPrimitive, Num, NumCast, One, Signed, ToPrimitive, Zero,
};
#[cfg(any(feature = "std", feature = "libm"))]
pub use num_traits::{Float, Pow};

#[cfg(feature = "rand")]
pub use impl_rand::{UniformNotNan, UniformOrdered};

/// A wrapper around floats providing implementations of `Eq`, `Ord`, and `Hash`.
///
/// NaN is sorted as *greater* than all other values and *equal*
/// to itself, in contradiction with the IEEE standard.
///
/// ```
/// use ordered_float::OrderedFloat;
/// use std::f32::NAN;
///
/// let mut v = [OrderedFloat(NAN), OrderedFloat(2.0), OrderedFloat(1.0)];
/// v.sort();
/// assert_eq!(v, [OrderedFloat(1.0), OrderedFloat(2.0), OrderedFloat(NAN)]);
/// ```
///
/// Because `OrderedFloat` implements `Ord` and `Eq`, it can be used as a key in a `HashSet`,
/// `HashMap`, `BTreeMap`, or `BTreeSet` (unlike the primitive `f32` or `f64` types):
///
/// ```
/// # use ordered_float::OrderedFloat;
/// # use std::collections::HashSet;
/// # use std::f32::NAN;
/// let mut s: HashSet<OrderedFloat<f32>> = HashSet::new();
/// s.insert(OrderedFloat(NAN));
/// assert!(s.contains(&OrderedFloat(NAN)));
/// ```
///
/// Some non-identical values are still considered equal by the [`PartialEq`] implementation,
/// and will therefore also be considered equal by maps, sets, and the `==` operator:
///
/// * `-0.0` and `+0.0` are considered equal.
///   This different sign may show up in printing, or when dividing by zero (the sign of the zero
///   becomes the sign of the resulting infinity).
/// * All NaN values are considered equal, even though they may have different
///   [bits](https://doc.rust-lang.org/std/primitive.f64.html#method.to_bits), and therefore
///   different [sign](https://doc.rust-lang.org/std/primitive.f64.html#method.is_sign_positive),
///   signaling/quiet status, and NaN payload bits.
///   
/// Therefore, `OrderedFloat` may be unsuitable for use as a key in interning and memoization
/// applications which require equal results from equal inputs, unless these cases make no
/// difference or are canonicalized before insertion.
///
/// # Representation
///
/// `OrderedFloat` has `#[repr(transparent)]` and permits any value, so it is sound to use
/// [transmute](core::mem::transmute) or pointer casts to convert between any type `T` and
/// `OrderedFloat<T>`.
/// However, consider using [`bytemuck`] as a safe alternative if possible.
///
#[cfg_attr(
    not(feature = "bytemuck"),
    doc = "[`bytemuck`]: https://docs.rs/bytemuck/1/"
)]
#[derive(Default, Clone, Copy)]
#[repr(transparent)]
pub struct OrderedFloat<T>(pub T);

#[cfg(feature = "derive-visitor")]
mod impl_derive_visitor {
    use crate::OrderedFloat;
    use derive_visitor::{Drive, DriveMut, Event, Visitor, VisitorMut};

    impl<T: 'static> Drive for OrderedFloat<T> {
        fn drive<V: Visitor>(&self, visitor: &mut V) {
            visitor.visit(self, Event::Enter);
            visitor.visit(self, Event::Exit);
        }
    }

    impl<T: 'static> DriveMut for OrderedFloat<T> {
        fn drive_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
            visitor.visit(self, Event::Enter);
            visitor.visit(self, Event::Exit);
        }
    }

    #[test]
    pub fn test_derive_visitor() {
        #[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
        pub enum Literal {
            Null,
            Float(OrderedFloat<f64>),
        }

        #[derive(Visitor, VisitorMut)]
        #[visitor(Literal(enter))]
        struct FloatExpr(bool);

        impl FloatExpr {
            fn enter_literal(&mut self, lit: &Literal) {
                if let Literal::Float(_) = lit {
                    self.0 = true;
                }
            }
        }

        assert!({
            let mut visitor = FloatExpr(false);
            Literal::Null.drive(&mut visitor);
            !visitor.0
        });

        assert!({
            let mut visitor = FloatExpr(false);
            Literal::Null.drive_mut(&mut visitor);
            !visitor.0
        });

        assert!({
            let mut visitor = FloatExpr(false);
            Literal::Float(OrderedFloat(0.0)).drive(&mut visitor);
            visitor.0
        });

        assert!({
            let mut visitor = FloatExpr(false);
            Literal::Float(OrderedFloat(0.0)).drive_mut(&mut visitor);
            visitor.0
        });
    }
}

#[cfg(feature = "num-cmp")]
mod impl_num_cmp {
    use super::OrderedFloat;
    use core::cmp::Ordering;
    use num_cmp::NumCmp;
    use num_traits::float::FloatCore;

    impl<T, U> NumCmp<U> for OrderedFloat<T>
    where
        T: FloatCore + NumCmp<U>,
        U: Copy,
    {
        fn num_cmp(self, other: U) -> Option<Ordering> {
            NumCmp::num_cmp(self.0, other)
        }

        fn num_eq(self, other: U) -> bool {
            NumCmp::num_eq(self.0, other)
        }

        fn num_ne(self, other: U) -> bool {
            NumCmp::num_ne(self.0, other)
        }

        fn num_lt(self, other: U) -> bool {
            NumCmp::num_lt(self.0, other)
        }

        fn num_gt(self, other: U) -> bool {
            NumCmp::num_gt(self.0, other)
        }

        fn num_le(self, other: U) -> bool {
            NumCmp::num_le(self.0, other)
        }

        fn num_ge(self, other: U) -> bool {
            NumCmp::num_ge(self.0, other)
        }
    }

    #[test]
    pub fn test_num_cmp() {
        let f = OrderedFloat(1.0);

        assert_eq!(NumCmp::num_cmp(f, 1.0), Some(Ordering::Equal));
        assert_eq!(NumCmp::num_cmp(f, -1.0), Some(Ordering::Greater));
        assert_eq!(NumCmp::num_cmp(f, 2.0), Some(Ordering::Less));

        assert!(NumCmp::num_eq(f, 1));
        assert!(NumCmp::num_ne(f, -1));
        assert!(NumCmp::num_lt(f, 100));
        assert!(NumCmp::num_gt(f, 0));
        assert!(NumCmp::num_le(f, 1));
        assert!(NumCmp::num_le(f, 2));
        assert!(NumCmp::num_ge(f, 1));
        assert!(NumCmp::num_ge(f, -1));
    }
}

impl<T: FloatCore> OrderedFloat<T> {
    /// Get the value out.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: FloatCore> AsRef<T> for OrderedFloat<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: FloatCore> AsMut<T> for OrderedFloat<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<'a, T: FloatCore> From<&'a T> for &'a OrderedFloat<T> {
    #[inline]
    fn from(t: &'a T) -> &'a OrderedFloat<T> {
        // Safety: OrderedFloat is #[repr(transparent)] and has no invalid values.
        unsafe { &*(t as *const T as *const OrderedFloat<T>) }
    }
}

impl<'a, T: FloatCore> From<&'a mut T> for &'a mut OrderedFloat<T> {
    #[inline]
    fn from(t: &'a mut T) -> &'a mut OrderedFloat<T> {
        // Safety: OrderedFloat is #[repr(transparent)] and has no invalid values.
        unsafe { &mut *(t as *mut T as *mut OrderedFloat<T>) }
    }
}

impl<T: FloatCore> PartialOrd for OrderedFloat<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        !self.ge(other)
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        other.ge(self)
    }

    #[inline]
    fn gt(&self, other: &Self) -> bool {
        !other.ge(self)
    }

    #[inline]
    fn ge(&self, other: &Self) -> bool {
        // We consider all NaNs equal, and NaN is the largest possible
        // value. Thus if self is NaN we always return true. Otherwise
        // self >= other is correct. If other is also not NaN it is trivially
        // correct, and if it is we note that nothing can be greater or
        // equal to NaN except NaN itself, which we already handled earlier.
        self.0.is_nan() | (self.0 >= other.0)
    }
}

impl<T: FloatCore> Ord for OrderedFloat<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        #[allow(clippy::comparison_chain)]
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl<T: FloatCore> PartialEq for OrderedFloat<T> {
    #[inline]
    fn eq(&self, other: &OrderedFloat<T>) -> bool {
        if self.0.is_nan() {
            other.0.is_nan()
        } else {
            self.0 == other.0
        }
    }
}

impl<T: FloatCore> PartialEq<T> for OrderedFloat<T> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<T: fmt::Debug> fmt::Debug for OrderedFloat<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: FloatCore + fmt::Display> fmt::Display for OrderedFloat<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: FloatCore + fmt::LowerExp> fmt::LowerExp for OrderedFloat<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: FloatCore + fmt::UpperExp> fmt::UpperExp for OrderedFloat<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<OrderedFloat<f32>> for f32 {
    #[inline]
    fn from(f: OrderedFloat<f32>) -> f32 {
        f.0
    }
}

impl From<OrderedFloat<f64>> for f64 {
    #[inline]
    fn from(f: OrderedFloat<f64>) -> f64 {
        f.0
    }
}

impl<T: FloatCore> From<T> for OrderedFloat<T> {
    #[inline]
    fn from(val: T) -> Self {
        OrderedFloat(val)
    }
}

impl From<bool> for OrderedFloat<f32> {
    fn from(val: bool) -> Self {
        OrderedFloat(val as u8 as f32)
    }
}

impl From<bool> for OrderedFloat<f64> {
    fn from(val: bool) -> Self {
        OrderedFloat(val as u8 as f64)
    }
}

macro_rules! impl_ordered_float_from {
    ($dst:ty, $src:ty) => {
        impl From<$src> for OrderedFloat<$dst> {
            fn from(val: $src) -> Self {
                OrderedFloat(val.into())
            }
        }
    };
}
impl_ordered_float_from! {f64, i8}
impl_ordered_float_from! {f64, i16}
impl_ordered_float_from! {f64, i32}
impl_ordered_float_from! {f64, u8}
impl_ordered_float_from! {f64, u16}
impl_ordered_float_from! {f64, u32}
impl_ordered_float_from! {f32, i8}
impl_ordered_float_from! {f32, i16}
impl_ordered_float_from! {f32, u8}
impl_ordered_float_from! {f32, u16}

impl<T: FloatCore> Deref for OrderedFloat<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: FloatCore> DerefMut for OrderedFloat<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: FloatCore> Eq for OrderedFloat<T> {}

macro_rules! impl_ordered_float_binop {
    ($imp:ident, $method:ident, $assign_imp:ident, $assign_method:ident) => {
        impl<T: $imp> $imp for OrderedFloat<T> {
            type Output = OrderedFloat<T::Output>;

            #[inline]
            fn $method(self, other: Self) -> Self::Output {
                OrderedFloat((self.0).$method(other.0))
            }
        }

        // Work around for: https://github.com/reem/rust-ordered-float/issues/91
        impl<'a, T: $imp + Copy> $imp<Self> for &'a OrderedFloat<T> {
            type Output = OrderedFloat<T::Output>;

            #[inline]
            fn $method(self, other: Self) -> Self::Output {
                OrderedFloat((self.0).$method(other.0))
            }
        }

        impl<T: $imp> $imp<T> for OrderedFloat<T> {
            type Output = OrderedFloat<T::Output>;

            #[inline]
            fn $method(self, other: T) -> Self::Output {
                OrderedFloat((self.0).$method(other))
            }
        }

        impl<'a, T> $imp<&'a T> for OrderedFloat<T>
        where
            T: $imp<&'a T>,
        {
            type Output = OrderedFloat<<T as $imp<&'a T>>::Output>;

            #[inline]
            fn $method(self, other: &'a T) -> Self::Output {
                OrderedFloat((self.0).$method(other))
            }
        }

        impl<'a, T> $imp<&'a Self> for OrderedFloat<T>
        where
            T: $imp<&'a T>,
        {
            type Output = OrderedFloat<<T as $imp<&'a T>>::Output>;

            #[inline]
            fn $method(self, other: &'a Self) -> Self::Output {
                OrderedFloat((self.0).$method(&other.0))
            }
        }

        impl<'a, T> $imp<OrderedFloat<T>> for &'a OrderedFloat<T>
        where
            &'a T: $imp<T>,
        {
            type Output = OrderedFloat<<&'a T as $imp<T>>::Output>;

            #[inline]
            fn $method(self, other: OrderedFloat<T>) -> Self::Output {
                OrderedFloat((self.0).$method(other.0))
            }
        }

        impl<'a, T> $imp<T> for &'a OrderedFloat<T>
        where
            &'a T: $imp<T>,
        {
            type Output = OrderedFloat<<&'a T as $imp<T>>::Output>;

            #[inline]
            fn $method(self, other: T) -> Self::Output {
                OrderedFloat((self.0).$method(other))
            }
        }

        impl<'a, T> $imp<&'a T> for &'a OrderedFloat<T>
        where
            &'a T: $imp,
        {
            type Output = OrderedFloat<<&'a T as $imp>::Output>;

            #[inline]
            fn $method(self, other: &'a T) -> Self::Output {
                OrderedFloat((self.0).$method(other))
            }
        }

        impl<T: $assign_imp> $assign_imp<T> for OrderedFloat<T> {
            #[inline]
            fn $assign_method(&mut self, other: T) {
                (self.0).$assign_method(other);
            }
        }

        impl<'a, T: $assign_imp<&'a T>> $assign_imp<&'a T> for OrderedFloat<T> {
            #[inline]
            fn $assign_method(&mut self, other: &'a T) {
                (self.0).$assign_method(other);
            }
        }

        impl<T: $assign_imp> $assign_imp for OrderedFloat<T> {
            #[inline]
            fn $assign_method(&mut self, other: Self) {
                (self.0).$assign_method(other.0);
            }
        }

        impl<'a, T: $assign_imp<&'a T>> $assign_imp<&'a Self> for OrderedFloat<T> {
            #[inline]
            fn $assign_method(&mut self, other: &'a Self) {
                (self.0).$assign_method(&other.0);
            }
        }
    };
}

impl_ordered_float_binop! {Add, add, AddAssign, add_assign}
impl_ordered_float_binop! {Sub, sub, SubAssign, sub_assign}
impl_ordered_float_binop! {Mul, mul, MulAssign, mul_assign}
impl_ordered_float_binop! {Div, div, DivAssign, div_assign}
impl_ordered_float_binop! {Rem, rem, RemAssign, rem_assign}

macro_rules! impl_ordered_float_pow {
    ($inner:ty, $rhs:ty) => {
        #[cfg(any(feature = "std", feature = "libm"))]
        impl Pow<$rhs> for OrderedFloat<$inner> {
            type Output = OrderedFloat<$inner>;
            #[inline]
            fn pow(self, rhs: $rhs) -> OrderedFloat<$inner> {
                OrderedFloat(<$inner>::pow(self.0, rhs))
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a> Pow<&'a $rhs> for OrderedFloat<$inner> {
            type Output = OrderedFloat<$inner>;
            #[inline]
            fn pow(self, rhs: &'a $rhs) -> OrderedFloat<$inner> {
                OrderedFloat(<$inner>::pow(self.0, *rhs))
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a> Pow<$rhs> for &'a OrderedFloat<$inner> {
            type Output = OrderedFloat<$inner>;
            #[inline]
            fn pow(self, rhs: $rhs) -> OrderedFloat<$inner> {
                OrderedFloat(<$inner>::pow(self.0, rhs))
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a, 'b> Pow<&'a $rhs> for &'b OrderedFloat<$inner> {
            type Output = OrderedFloat<$inner>;
            #[inline]
            fn pow(self, rhs: &'a $rhs) -> OrderedFloat<$inner> {
                OrderedFloat(<$inner>::pow(self.0, *rhs))
            }
        }
    };
}

impl_ordered_float_pow! {f32, i8}
impl_ordered_float_pow! {f32, i16}
impl_ordered_float_pow! {f32, u8}
impl_ordered_float_pow! {f32, u16}
impl_ordered_float_pow! {f32, i32}
impl_ordered_float_pow! {f64, i8}
impl_ordered_float_pow! {f64, i16}
impl_ordered_float_pow! {f64, u8}
impl_ordered_float_pow! {f64, u16}
impl_ordered_float_pow! {f64, i32}
impl_ordered_float_pow! {f32, f32}
impl_ordered_float_pow! {f64, f32}
impl_ordered_float_pow! {f64, f64}

macro_rules! impl_ordered_float_self_pow {
    ($base:ty, $exp:ty) => {
        #[cfg(any(feature = "std", feature = "libm"))]
        impl Pow<OrderedFloat<$exp>> for OrderedFloat<$base> {
            type Output = OrderedFloat<$base>;
            #[inline]
            fn pow(self, rhs: OrderedFloat<$exp>) -> OrderedFloat<$base> {
                OrderedFloat(<$base>::pow(self.0, rhs.0))
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a> Pow<&'a OrderedFloat<$exp>> for OrderedFloat<$base> {
            type Output = OrderedFloat<$base>;
            #[inline]
            fn pow(self, rhs: &'a OrderedFloat<$exp>) -> OrderedFloat<$base> {
                OrderedFloat(<$base>::pow(self.0, rhs.0))
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a> Pow<OrderedFloat<$exp>> for &'a OrderedFloat<$base> {
            type Output = OrderedFloat<$base>;
            #[inline]
            fn pow(self, rhs: OrderedFloat<$exp>) -> OrderedFloat<$base> {
                OrderedFloat(<$base>::pow(self.0, rhs.0))
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a, 'b> Pow<&'a OrderedFloat<$exp>> for &'b OrderedFloat<$base> {
            type Output = OrderedFloat<$base>;
            #[inline]
            fn pow(self, rhs: &'a OrderedFloat<$exp>) -> OrderedFloat<$base> {
                OrderedFloat(<$base>::pow(self.0, rhs.0))
            }
        }
    };
}

impl_ordered_float_self_pow! {f32, f32}
impl_ordered_float_self_pow! {f64, f32}
impl_ordered_float_self_pow! {f64, f64}

/// Adds a float directly.
impl<T: FloatCore + Sum> Sum for OrderedFloat<T> {
    fn sum<I: Iterator<Item = OrderedFloat<T>>>(iter: I) -> Self {
        OrderedFloat(iter.map(|v| v.0).sum())
    }
}

impl<'a, T: FloatCore + Sum + 'a> Sum<&'a OrderedFloat<T>> for OrderedFloat<T> {
    #[inline]
    fn sum<I: Iterator<Item = &'a OrderedFloat<T>>>(iter: I) -> Self {
        iter.cloned().sum()
    }
}

impl<T: FloatCore + Product> Product for OrderedFloat<T> {
    fn product<I: Iterator<Item = OrderedFloat<T>>>(iter: I) -> Self {
        OrderedFloat(iter.map(|v| v.0).product())
    }
}

impl<'a, T: FloatCore + Product + 'a> Product<&'a OrderedFloat<T>> for OrderedFloat<T> {
    #[inline]
    fn product<I: Iterator<Item = &'a OrderedFloat<T>>>(iter: I) -> Self {
        iter.cloned().product()
    }
}

impl<T: FloatCore + Signed> Signed for OrderedFloat<T> {
    #[inline]
    fn abs(&self) -> Self {
        OrderedFloat(self.0.abs())
    }

    fn abs_sub(&self, other: &Self) -> Self {
        OrderedFloat(Signed::abs_sub(&self.0, &other.0))
    }

    #[inline]
    fn signum(&self) -> Self {
        OrderedFloat(self.0.signum())
    }
    #[inline]
    fn is_positive(&self) -> bool {
        self.0.is_positive()
    }
    #[inline]
    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

impl<T: Bounded> Bounded for OrderedFloat<T> {
    #[inline]
    fn min_value() -> Self {
        OrderedFloat(T::min_value())
    }

    #[inline]
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

    #[inline]
    fn neg(self) -> Self::Output {
        OrderedFloat(-self.0)
    }
}

impl<'a, T> Neg for &'a OrderedFloat<T>
where
    &'a T: Neg,
{
    type Output = OrderedFloat<<&'a T as Neg>::Output>;

    #[inline]
    fn neg(self) -> Self::Output {
        OrderedFloat(-(&self.0))
    }
}

impl<T: Zero> Zero for OrderedFloat<T> {
    #[inline]
    fn zero() -> Self {
        OrderedFloat(T::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T: One> One for OrderedFloat<T> {
    #[inline]
    fn one() -> Self {
        OrderedFloat(T::one())
    }
}

impl<T: NumCast> NumCast for OrderedFloat<T> {
    #[inline]
    fn from<F: ToPrimitive>(n: F) -> Option<Self> {
        T::from(n).map(OrderedFloat)
    }
}

macro_rules! impl_as_primitive {
    (@ (NotNan<$T: ty>) => $(#[$cfg:meta])* impl (NotNan<$U: ty>) ) => {
        $(#[$cfg])*
        impl AsPrimitive<NotNan<$U>> for NotNan<$T> {
            #[inline] fn as_(self) -> NotNan<$U> {
                // Safety: `NotNan` guarantees that the value is not NaN.
                unsafe {NotNan::new_unchecked(self.0 as $U) }
            }
        }
    };
    (@ ($T: ty) => $(#[$cfg:meta])* impl (NotNan<$U: ty>) ) => {
        $(#[$cfg])*
        impl AsPrimitive<NotNan<$U>> for $T {
            #[inline] fn as_(self) -> NotNan<$U> { NotNan(self as $U) }
        }
    };
    (@ (NotNan<$T: ty>) => $(#[$cfg:meta])* impl ($U: ty) ) => {
        $(#[$cfg])*
        impl AsPrimitive<$U> for NotNan<$T> {
            #[inline] fn as_(self) -> $U { self.0 as $U }
        }
    };
    (@ (OrderedFloat<$T: ty>) => $(#[$cfg:meta])* impl (OrderedFloat<$U: ty>) ) => {
        $(#[$cfg])*
        impl AsPrimitive<OrderedFloat<$U>> for OrderedFloat<$T> {
            #[inline] fn as_(self) -> OrderedFloat<$U> { OrderedFloat(self.0 as $U) }
        }
    };
    (@ ($T: ty) => $(#[$cfg:meta])* impl (OrderedFloat<$U: ty>) ) => {
        $(#[$cfg])*
        impl AsPrimitive<OrderedFloat<$U>> for $T {
            #[inline] fn as_(self) -> OrderedFloat<$U> { OrderedFloat(self as $U) }
        }
    };
    (@ (OrderedFloat<$T: ty>) => $(#[$cfg:meta])* impl ($U: ty) ) => {
        $(#[$cfg])*
        impl AsPrimitive<$U> for OrderedFloat<$T> {
            #[inline] fn as_(self) -> $U { self.0 as $U }
        }
    };
    ($T: tt => { $( $U: tt ),* } ) => {$(
        impl_as_primitive!(@ $T => impl $U);
    )*};
}

impl_as_primitive!((OrderedFloat<f32>) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((OrderedFloat<f64>) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });

impl_as_primitive!((NotNan<f32>) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((NotNan<f64>) => { (NotNan<f32>), (NotNan<f64>) });

impl_as_primitive!((u8) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((i8) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((u16) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((i16) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((u32) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((i32) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((u64) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((i64) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((usize) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((isize) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((f32) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });
impl_as_primitive!((f64) => { (OrderedFloat<f32>), (OrderedFloat<f64>) });

impl_as_primitive!((u8) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((i8) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((u16) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((i16) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((u32) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((i32) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((u64) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((i64) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((usize) => { (NotNan<f32>), (NotNan<f64>) });
impl_as_primitive!((isize) => { (NotNan<f32>), (NotNan<f64>) });

impl_as_primitive!((OrderedFloat<f32>) => { (u8), (u16), (u32), (u64), (usize), (i8), (i16), (i32), (i64), (isize), (f32), (f64) });
impl_as_primitive!((OrderedFloat<f64>) => { (u8), (u16), (u32), (u64), (usize), (i8), (i16), (i32), (i64), (isize), (f32), (f64) });

impl_as_primitive!((NotNan<f32>) => { (u8), (u16), (u32), (u64), (usize), (i8), (i16), (i32), (i64), (isize), (f32), (f64) });
impl_as_primitive!((NotNan<f64>) => { (u8), (u16), (u32), (u64), (usize), (i8), (i16), (i32), (i64), (isize), (f32), (f64) });

impl<T: FromPrimitive> FromPrimitive for OrderedFloat<T> {
    fn from_i64(n: i64) -> Option<Self> {
        T::from_i64(n).map(OrderedFloat)
    }
    fn from_u64(n: u64) -> Option<Self> {
        T::from_u64(n).map(OrderedFloat)
    }
    fn from_isize(n: isize) -> Option<Self> {
        T::from_isize(n).map(OrderedFloat)
    }
    fn from_i8(n: i8) -> Option<Self> {
        T::from_i8(n).map(OrderedFloat)
    }
    fn from_i16(n: i16) -> Option<Self> {
        T::from_i16(n).map(OrderedFloat)
    }
    fn from_i32(n: i32) -> Option<Self> {
        T::from_i32(n).map(OrderedFloat)
    }
    fn from_usize(n: usize) -> Option<Self> {
        T::from_usize(n).map(OrderedFloat)
    }
    fn from_u8(n: u8) -> Option<Self> {
        T::from_u8(n).map(OrderedFloat)
    }
    fn from_u16(n: u16) -> Option<Self> {
        T::from_u16(n).map(OrderedFloat)
    }
    fn from_u32(n: u32) -> Option<Self> {
        T::from_u32(n).map(OrderedFloat)
    }
    fn from_f32(n: f32) -> Option<Self> {
        T::from_f32(n).map(OrderedFloat)
    }
    fn from_f64(n: f64) -> Option<Self> {
        T::from_f64(n).map(OrderedFloat)
    }
}

impl<T: ToPrimitive> ToPrimitive for OrderedFloat<T> {
    fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }
    fn to_u64(&self) -> Option<u64> {
        self.0.to_u64()
    }
    fn to_isize(&self) -> Option<isize> {
        self.0.to_isize()
    }
    fn to_i8(&self) -> Option<i8> {
        self.0.to_i8()
    }
    fn to_i16(&self) -> Option<i16> {
        self.0.to_i16()
    }
    fn to_i32(&self) -> Option<i32> {
        self.0.to_i32()
    }
    fn to_usize(&self) -> Option<usize> {
        self.0.to_usize()
    }
    fn to_u8(&self) -> Option<u8> {
        self.0.to_u8()
    }
    fn to_u16(&self) -> Option<u16> {
        self.0.to_u16()
    }
    fn to_u32(&self) -> Option<u32> {
        self.0.to_u32()
    }
    fn to_f32(&self) -> Option<f32> {
        self.0.to_f32()
    }
    fn to_f64(&self) -> Option<f64> {
        self.0.to_f64()
    }
}

impl<T: FloatCore> FloatCore for OrderedFloat<T> {
    fn nan() -> Self {
        OrderedFloat(T::nan())
    }
    fn infinity() -> Self {
        OrderedFloat(T::infinity())
    }
    fn neg_infinity() -> Self {
        OrderedFloat(T::neg_infinity())
    }
    fn neg_zero() -> Self {
        OrderedFloat(T::neg_zero())
    }
    fn min_value() -> Self {
        OrderedFloat(T::min_value())
    }
    fn min_positive_value() -> Self {
        OrderedFloat(T::min_positive_value())
    }
    fn max_value() -> Self {
        OrderedFloat(T::max_value())
    }
    fn is_nan(self) -> bool {
        self.0.is_nan()
    }
    fn is_infinite(self) -> bool {
        self.0.is_infinite()
    }
    fn is_finite(self) -> bool {
        self.0.is_finite()
    }
    fn is_normal(self) -> bool {
        self.0.is_normal()
    }
    fn classify(self) -> FpCategory {
        self.0.classify()
    }
    fn floor(self) -> Self {
        OrderedFloat(self.0.floor())
    }
    fn ceil(self) -> Self {
        OrderedFloat(self.0.ceil())
    }
    fn round(self) -> Self {
        OrderedFloat(self.0.round())
    }
    fn trunc(self) -> Self {
        OrderedFloat(self.0.trunc())
    }
    fn fract(self) -> Self {
        OrderedFloat(self.0.fract())
    }
    fn abs(self) -> Self {
        OrderedFloat(self.0.abs())
    }
    fn signum(self) -> Self {
        OrderedFloat(self.0.signum())
    }
    fn is_sign_positive(self) -> bool {
        self.0.is_sign_positive()
    }
    fn is_sign_negative(self) -> bool {
        self.0.is_sign_negative()
    }
    fn recip(self) -> Self {
        OrderedFloat(self.0.recip())
    }
    fn powi(self, n: i32) -> Self {
        OrderedFloat(self.0.powi(n))
    }
    fn integer_decode(self) -> (u64, i16, i8) {
        self.0.integer_decode()
    }
    fn epsilon() -> Self {
        OrderedFloat(T::epsilon())
    }
    fn to_degrees(self) -> Self {
        OrderedFloat(self.0.to_degrees())
    }
    fn to_radians(self) -> Self {
        OrderedFloat(self.0.to_radians())
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
impl<T: Float + FloatCore> Float for OrderedFloat<T> {
    fn nan() -> Self {
        OrderedFloat(<T as Float>::nan())
    }
    fn infinity() -> Self {
        OrderedFloat(<T as Float>::infinity())
    }
    fn neg_infinity() -> Self {
        OrderedFloat(<T as Float>::neg_infinity())
    }
    fn neg_zero() -> Self {
        OrderedFloat(<T as Float>::neg_zero())
    }
    fn min_value() -> Self {
        OrderedFloat(<T as Float>::min_value())
    }
    fn min_positive_value() -> Self {
        OrderedFloat(<T as Float>::min_positive_value())
    }
    fn max_value() -> Self {
        OrderedFloat(<T as Float>::max_value())
    }
    fn is_nan(self) -> bool {
        Float::is_nan(self.0)
    }
    fn is_infinite(self) -> bool {
        Float::is_infinite(self.0)
    }
    fn is_finite(self) -> bool {
        Float::is_finite(self.0)
    }
    fn is_normal(self) -> bool {
        Float::is_normal(self.0)
    }
    fn classify(self) -> FpCategory {
        Float::classify(self.0)
    }
    fn floor(self) -> Self {
        OrderedFloat(Float::floor(self.0))
    }
    fn ceil(self) -> Self {
        OrderedFloat(Float::ceil(self.0))
    }
    fn round(self) -> Self {
        OrderedFloat(Float::round(self.0))
    }
    fn trunc(self) -> Self {
        OrderedFloat(Float::trunc(self.0))
    }
    fn fract(self) -> Self {
        OrderedFloat(Float::fract(self.0))
    }
    fn abs(self) -> Self {
        OrderedFloat(Float::abs(self.0))
    }
    fn signum(self) -> Self {
        OrderedFloat(Float::signum(self.0))
    }
    fn is_sign_positive(self) -> bool {
        Float::is_sign_positive(self.0)
    }
    fn is_sign_negative(self) -> bool {
        Float::is_sign_negative(self.0)
    }
    fn mul_add(self, a: Self, b: Self) -> Self {
        OrderedFloat(self.0.mul_add(a.0, b.0))
    }
    fn recip(self) -> Self {
        OrderedFloat(Float::recip(self.0))
    }
    fn powi(self, n: i32) -> Self {
        OrderedFloat(Float::powi(self.0, n))
    }
    fn powf(self, n: Self) -> Self {
        OrderedFloat(self.0.powf(n.0))
    }
    fn sqrt(self) -> Self {
        OrderedFloat(self.0.sqrt())
    }
    fn exp(self) -> Self {
        OrderedFloat(self.0.exp())
    }
    fn exp2(self) -> Self {
        OrderedFloat(self.0.exp2())
    }
    fn ln(self) -> Self {
        OrderedFloat(self.0.ln())
    }
    fn log(self, base: Self) -> Self {
        OrderedFloat(self.0.log(base.0))
    }
    fn log2(self) -> Self {
        OrderedFloat(self.0.log2())
    }
    fn log10(self) -> Self {
        OrderedFloat(self.0.log10())
    }
    fn max(self, other: Self) -> Self {
        OrderedFloat(Float::max(self.0, other.0))
    }
    fn min(self, other: Self) -> Self {
        OrderedFloat(Float::min(self.0, other.0))
    }
    fn abs_sub(self, other: Self) -> Self {
        OrderedFloat(self.0.abs_sub(other.0))
    }
    fn cbrt(self) -> Self {
        OrderedFloat(self.0.cbrt())
    }
    fn hypot(self, other: Self) -> Self {
        OrderedFloat(self.0.hypot(other.0))
    }
    fn sin(self) -> Self {
        OrderedFloat(self.0.sin())
    }
    fn cos(self) -> Self {
        OrderedFloat(self.0.cos())
    }
    fn tan(self) -> Self {
        OrderedFloat(self.0.tan())
    }
    fn asin(self) -> Self {
        OrderedFloat(self.0.asin())
    }
    fn acos(self) -> Self {
        OrderedFloat(self.0.acos())
    }
    fn atan(self) -> Self {
        OrderedFloat(self.0.atan())
    }
    fn atan2(self, other: Self) -> Self {
        OrderedFloat(self.0.atan2(other.0))
    }
    fn sin_cos(self) -> (Self, Self) {
        let (a, b) = self.0.sin_cos();
        (OrderedFloat(a), OrderedFloat(b))
    }
    fn exp_m1(self) -> Self {
        OrderedFloat(self.0.exp_m1())
    }
    fn ln_1p(self) -> Self {
        OrderedFloat(self.0.ln_1p())
    }
    fn sinh(self) -> Self {
        OrderedFloat(self.0.sinh())
    }
    fn cosh(self) -> Self {
        OrderedFloat(self.0.cosh())
    }
    fn tanh(self) -> Self {
        OrderedFloat(self.0.tanh())
    }
    fn asinh(self) -> Self {
        OrderedFloat(self.0.asinh())
    }
    fn acosh(self) -> Self {
        OrderedFloat(self.0.acosh())
    }
    fn atanh(self) -> Self {
        OrderedFloat(self.0.atanh())
    }
    fn integer_decode(self) -> (u64, i16, i8) {
        Float::integer_decode(self.0)
    }
    fn epsilon() -> Self {
        OrderedFloat(<T as Float>::epsilon())
    }
    fn to_degrees(self) -> Self {
        OrderedFloat(Float::to_degrees(self.0))
    }
    fn to_radians(self) -> Self {
        OrderedFloat(Float::to_radians(self.0))
    }
}

impl<T: FloatCore + Num> Num for OrderedFloat<T> {
    type FromStrRadixErr = T::FromStrRadixErr;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        T::from_str_radix(str, radix).map(OrderedFloat)
    }
}

/// A wrapper around floats providing an implementation of `Eq`, `Ord` and `Hash`.
///
/// A NaN value cannot be stored in this type.
///
/// ```
/// use ordered_float::NotNan;
///
/// let mut v = [NotNan::new(2.0).unwrap(), NotNan::new(1.0).unwrap()];
/// v.sort();
/// assert_eq!(v, [1.0, 2.0]);
/// ```
///
/// Because `NotNan` implements `Ord` and `Eq`, it can be used as a key in a `HashSet`,
/// `HashMap`, `BTreeMap`, or `BTreeSet` (unlike the primitive `f32` or `f64` types):
///
/// ```
/// # use ordered_float::NotNan;
/// # use std::collections::HashSet;
/// let mut s: HashSet<NotNan<f32>> = HashSet::new();
/// let key = NotNan::new(1.0).unwrap();
/// s.insert(key);
/// assert!(s.contains(&key));
/// ```
///
/// `-0.0` and `+0.0` are still considered equal. This different sign may show up in printing,
/// or when dividing by zero (the sign of the zero becomes the sign of the resulting infinity).
/// Therefore, `NotNan` may be unsuitable for use as a key in interning and memoization
/// applications which require equal results from equal inputs, unless signed zeros make no
/// difference or are canonicalized before insertion.
///
/// Arithmetic on NotNan values will panic if it produces a NaN value:
///
/// ```should_panic
/// # use ordered_float::NotNan;
/// let a = NotNan::new(std::f32::INFINITY).unwrap();
/// let b = NotNan::new(std::f32::NEG_INFINITY).unwrap();
///
/// // This will panic:
/// let c = a + b;
/// ```
///
/// # Representation
///
/// `NotNan` has `#[repr(transparent)]`, so it is sound to use
/// [transmute](core::mem::transmute) or pointer casts to convert between any type `T` and
/// `NotNan<T>`, as long as this does not create a NaN value.
/// However, consider using [`bytemuck`] as a safe alternative if possible.
#[cfg_attr(
    not(feature = "bytemuck"),
    doc = "[`bytemuck`]: https://docs.rs/bytemuck/1/"
)]
#[derive(PartialOrd, PartialEq, Default, Clone, Copy)]
#[repr(transparent)]
pub struct NotNan<T>(T);

impl<T: FloatCore> NotNan<T> {
    /// Create a `NotNan` value.
    ///
    /// Returns `Err` if `val` is NaN
    pub fn new(val: T) -> Result<Self, FloatIsNan> {
        match val {
            ref val if val.is_nan() => Err(FloatIsNan),
            val => Ok(NotNan(val)),
        }
    }
}

impl<T> NotNan<T> {
    /// Get the value out.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Create a `NotNan` value from a value that is guaranteed to not be NaN
    ///
    /// # Safety
    ///
    /// Behaviour is undefined if `val` is NaN
    #[inline]
    pub const unsafe fn new_unchecked(val: T) -> Self {
        NotNan(val)
    }

    /// Create a `NotNan` value from a value that is guaranteed to not be NaN
    ///
    /// # Safety
    ///
    /// Behaviour is undefined if `val` is NaN
    #[deprecated(
        since = "2.5.0",
        note = "Please use the new_unchecked function instead."
    )]
    #[inline]
    pub const unsafe fn unchecked_new(val: T) -> Self {
        Self::new_unchecked(val)
    }
}

impl<T: FloatCore> AsRef<T> for NotNan<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl Borrow<f32> for NotNan<f32> {
    #[inline]
    fn borrow(&self) -> &f32 {
        &self.0
    }
}

impl Borrow<f64> for NotNan<f64> {
    #[inline]
    fn borrow(&self) -> &f64 {
        &self.0
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl<T: FloatCore> Ord for NotNan<T> {
    fn cmp(&self, other: &NotNan<T>) -> Ordering {
        // Can't use unreachable_unchecked because unsafe code can't depend on FloatCore impl.
        // https://github.com/reem/rust-ordered-float/issues/150
        self.partial_cmp(other)
            .expect("partial_cmp failed for non-NaN value")
    }
}

impl<T: fmt::Debug> fmt::Debug for NotNan<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: FloatCore + fmt::Display> fmt::Display for NotNan<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl NotNan<f64> {
    /// Converts this [`NotNan`]`<`[`f64`]`>` to a [`NotNan`]`<`[`f32`]`>` while giving up on
    /// precision, [using `roundTiesToEven` as rounding mode, yielding `Infinity` on
    /// overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics).
    ///
    /// Note: For the reverse conversion (from `NotNan<f32>` to `NotNan<f64>`), you can use
    /// `.into()`.
    pub fn as_f32(self) -> NotNan<f32> {
        // This is not destroying invariants, as it is a pure rounding operation. The only two
        // special cases are where f32 would be overflowing, then the operation yields
        // Infinity, or where the input is already NaN, in which case the invariant is
        // already broken elsewhere.
        NotNan(self.0 as f32)
    }
}

impl From<NotNan<f32>> for f32 {
    #[inline]
    fn from(value: NotNan<f32>) -> Self {
        value.0
    }
}

impl From<NotNan<f64>> for f64 {
    #[inline]
    fn from(value: NotNan<f64>) -> Self {
        value.0
    }
}

impl TryFrom<f32> for NotNan<f32> {
    type Error = FloatIsNan;
    #[inline]
    fn try_from(v: f32) -> Result<Self, Self::Error> {
        NotNan::new(v)
    }
}

impl TryFrom<f64> for NotNan<f64> {
    type Error = FloatIsNan;
    #[inline]
    fn try_from(v: f64) -> Result<Self, Self::Error> {
        NotNan::new(v)
    }
}

macro_rules! impl_from_int_primitive {
    ($primitive:ty, $inner:ty) => {
        impl From<$primitive> for NotNan<$inner> {
            fn from(source: $primitive) -> Self {
                // the primitives with which this macro will be called cannot hold a value that
                // f64::from would convert to NaN, so this does not hurt invariants
                NotNan(<$inner as From<$primitive>>::from(source))
            }
        }
    };
}

impl_from_int_primitive!(i8, f64);
impl_from_int_primitive!(i16, f64);
impl_from_int_primitive!(i32, f64);
impl_from_int_primitive!(u8, f64);
impl_from_int_primitive!(u16, f64);
impl_from_int_primitive!(u32, f64);

impl_from_int_primitive!(i8, f32);
impl_from_int_primitive!(i16, f32);
impl_from_int_primitive!(u8, f32);
impl_from_int_primitive!(u16, f32);

impl From<NotNan<f32>> for NotNan<f64> {
    #[inline]
    fn from(v: NotNan<f32>) -> NotNan<f64> {
        unsafe { NotNan::new_unchecked(v.0 as f64) }
    }
}

impl<T: FloatCore> Deref for NotNan<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: FloatCore + PartialEq> Eq for NotNan<T> {}

impl<T: FloatCore> PartialEq<T> for NotNan<T> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

/// Adds a float directly.
///
/// This returns a `T` and not a `NotNan<T>` because if the added value is NaN, this will be NaN
impl<T: FloatCore> Add<T> for NotNan<T> {
    type Output = T;

    #[inline]
    fn add(self, other: T) -> Self::Output {
        self.0 + other
    }
}

/// Adds a float directly.
///
/// Panics if the provided value is NaN.
impl<T: FloatCore + Sum> Sum for NotNan<T> {
    fn sum<I: Iterator<Item = NotNan<T>>>(iter: I) -> Self {
        NotNan::new(iter.map(|v| v.0).sum()).expect("Sum resulted in NaN")
    }
}

impl<'a, T: FloatCore + Sum + 'a> Sum<&'a NotNan<T>> for NotNan<T> {
    #[inline]
    fn sum<I: Iterator<Item = &'a NotNan<T>>>(iter: I) -> Self {
        iter.cloned().sum()
    }
}

/// Subtracts a float directly.
///
/// This returns a `T` and not a `NotNan<T>` because if the substracted value is NaN, this will be
/// NaN
impl<T: FloatCore> Sub<T> for NotNan<T> {
    type Output = T;

    #[inline]
    fn sub(self, other: T) -> Self::Output {
        self.0 - other
    }
}

/// Multiplies a float directly.
///
/// This returns a `T` and not a `NotNan<T>` because if the multiplied value is NaN, this will be
/// NaN
impl<T: FloatCore> Mul<T> for NotNan<T> {
    type Output = T;

    #[inline]
    fn mul(self, other: T) -> Self::Output {
        self.0 * other
    }
}

impl<T: FloatCore + Product> Product for NotNan<T> {
    fn product<I: Iterator<Item = NotNan<T>>>(iter: I) -> Self {
        NotNan::new(iter.map(|v| v.0).product()).expect("Product resulted in NaN")
    }
}

impl<'a, T: FloatCore + Product + 'a> Product<&'a NotNan<T>> for NotNan<T> {
    #[inline]
    fn product<I: Iterator<Item = &'a NotNan<T>>>(iter: I) -> Self {
        iter.cloned().product()
    }
}

/// Divides a float directly.
///
/// This returns a `T` and not a `NotNan<T>` because if the divided-by value is NaN, this will be
/// NaN
impl<T: FloatCore> Div<T> for NotNan<T> {
    type Output = T;

    #[inline]
    fn div(self, other: T) -> Self::Output {
        self.0 / other
    }
}

/// Calculates `%` with a float directly.
///
/// This returns a `T` and not a `NotNan<T>` because if the RHS is NaN, this will be NaN
impl<T: FloatCore> Rem<T> for NotNan<T> {
    type Output = T;

    #[inline]
    fn rem(self, other: T) -> Self::Output {
        self.0 % other
    }
}

macro_rules! impl_not_nan_binop {
    ($imp:ident, $method:ident, $assign_imp:ident, $assign_method:ident) => {
        impl<T: FloatCore> $imp for NotNan<T> {
            type Output = Self;

            #[inline]
            fn $method(self, other: Self) -> Self {
                NotNan::new(self.0.$method(other.0))
                    .expect("Operation on two NotNan resulted in NaN")
            }
        }

        impl<T: FloatCore> $imp<&T> for NotNan<T> {
            type Output = T;

            #[inline]
            fn $method(self, other: &T) -> Self::Output {
                self.$method(*other)
            }
        }

        impl<T: FloatCore> $imp<&Self> for NotNan<T> {
            type Output = NotNan<T>;

            #[inline]
            fn $method(self, other: &Self) -> Self::Output {
                self.$method(*other)
            }
        }

        impl<T: FloatCore> $imp for &NotNan<T> {
            type Output = NotNan<T>;

            #[inline]
            fn $method(self, other: Self) -> Self::Output {
                (*self).$method(*other)
            }
        }

        impl<T: FloatCore> $imp<NotNan<T>> for &NotNan<T> {
            type Output = NotNan<T>;

            #[inline]
            fn $method(self, other: NotNan<T>) -> Self::Output {
                (*self).$method(other)
            }
        }

        impl<T: FloatCore> $imp<T> for &NotNan<T> {
            type Output = T;

            #[inline]
            fn $method(self, other: T) -> Self::Output {
                (*self).$method(other)
            }
        }

        impl<T: FloatCore> $imp<&T> for &NotNan<T> {
            type Output = T;

            #[inline]
            fn $method(self, other: &T) -> Self::Output {
                (*self).$method(*other)
            }
        }

        impl<T: FloatCore + $assign_imp> $assign_imp for NotNan<T> {
            #[inline]
            fn $assign_method(&mut self, other: Self) {
                *self = (*self).$method(other);
            }
        }

        impl<T: FloatCore + $assign_imp> $assign_imp<&Self> for NotNan<T> {
            #[inline]
            fn $assign_method(&mut self, other: &Self) {
                *self = (*self).$method(*other);
            }
        }
    };
}

impl_not_nan_binop! {Add, add, AddAssign, add_assign}
impl_not_nan_binop! {Sub, sub, SubAssign, sub_assign}
impl_not_nan_binop! {Mul, mul, MulAssign, mul_assign}
impl_not_nan_binop! {Div, div, DivAssign, div_assign}
impl_not_nan_binop! {Rem, rem, RemAssign, rem_assign}

// Will panic if NaN value is return from the operation
macro_rules! impl_not_nan_pow {
    ($inner:ty, $rhs:ty) => {
        #[cfg(any(feature = "std", feature = "libm"))]
        impl Pow<$rhs> for NotNan<$inner> {
            type Output = NotNan<$inner>;
            #[inline]
            fn pow(self, rhs: $rhs) -> NotNan<$inner> {
                NotNan::new(<$inner>::pow(self.0, rhs)).expect("Pow resulted in NaN")
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a> Pow<&'a $rhs> for NotNan<$inner> {
            type Output = NotNan<$inner>;
            #[inline]
            fn pow(self, rhs: &'a $rhs) -> NotNan<$inner> {
                NotNan::new(<$inner>::pow(self.0, *rhs)).expect("Pow resulted in NaN")
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a> Pow<$rhs> for &'a NotNan<$inner> {
            type Output = NotNan<$inner>;
            #[inline]
            fn pow(self, rhs: $rhs) -> NotNan<$inner> {
                NotNan::new(<$inner>::pow(self.0, rhs)).expect("Pow resulted in NaN")
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a, 'b> Pow<&'a $rhs> for &'b NotNan<$inner> {
            type Output = NotNan<$inner>;
            #[inline]
            fn pow(self, rhs: &'a $rhs) -> NotNan<$inner> {
                NotNan::new(<$inner>::pow(self.0, *rhs)).expect("Pow resulted in NaN")
            }
        }
    };
}

impl_not_nan_pow! {f32, i8}
impl_not_nan_pow! {f32, i16}
impl_not_nan_pow! {f32, u8}
impl_not_nan_pow! {f32, u16}
impl_not_nan_pow! {f32, i32}
impl_not_nan_pow! {f64, i8}
impl_not_nan_pow! {f64, i16}
impl_not_nan_pow! {f64, u8}
impl_not_nan_pow! {f64, u16}
impl_not_nan_pow! {f64, i32}
impl_not_nan_pow! {f32, f32}
impl_not_nan_pow! {f64, f32}
impl_not_nan_pow! {f64, f64}

// This also should panic on NaN
macro_rules! impl_not_nan_self_pow {
    ($base:ty, $exp:ty) => {
        #[cfg(any(feature = "std", feature = "libm"))]
        impl Pow<NotNan<$exp>> for NotNan<$base> {
            type Output = NotNan<$base>;
            #[inline]
            fn pow(self, rhs: NotNan<$exp>) -> NotNan<$base> {
                NotNan::new(self.0.pow(rhs.0)).expect("Pow resulted in NaN")
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a> Pow<&'a NotNan<$exp>> for NotNan<$base> {
            type Output = NotNan<$base>;
            #[inline]
            fn pow(self, rhs: &'a NotNan<$exp>) -> NotNan<$base> {
                NotNan::new(self.0.pow(rhs.0)).expect("Pow resulted in NaN")
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a> Pow<NotNan<$exp>> for &'a NotNan<$base> {
            type Output = NotNan<$base>;
            #[inline]
            fn pow(self, rhs: NotNan<$exp>) -> NotNan<$base> {
                NotNan::new(self.0.pow(rhs.0)).expect("Pow resulted in NaN")
            }
        }

        #[cfg(any(feature = "std", feature = "libm"))]
        impl<'a, 'b> Pow<&'a NotNan<$exp>> for &'b NotNan<$base> {
            type Output = NotNan<$base>;
            #[inline]
            fn pow(self, rhs: &'a NotNan<$exp>) -> NotNan<$base> {
                NotNan::new(self.0.pow(rhs.0)).expect("Pow resulted in NaN")
            }
        }
    };
}

impl_not_nan_self_pow! {f32, f32}
impl_not_nan_self_pow! {f64, f32}
impl_not_nan_self_pow! {f64, f64}

impl<T: FloatCore> Neg for NotNan<T> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        NotNan(-self.0)
    }
}

impl<T: FloatCore> Neg for &NotNan<T> {
    type Output = NotNan<T>;

    #[inline]
    fn neg(self) -> Self::Output {
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
impl From<FloatIsNan> for std::io::Error {
    #[inline]
    fn from(e: FloatIsNan) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    }
}

impl<T: FloatCore> Zero for NotNan<T> {
    #[inline]
    fn zero() -> Self {
        NotNan(T::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T: FloatCore> One for NotNan<T> {
    #[inline]
    fn one() -> Self {
        NotNan(T::one())
    }
}

impl<T: FloatCore> Bounded for NotNan<T> {
    #[inline]
    fn min_value() -> Self {
        NotNan(T::min_value())
    }

    #[inline]
    fn max_value() -> Self {
        NotNan(T::max_value())
    }
}

impl<T: FloatCore + FromStr> FromStr for NotNan<T> {
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

impl<T: FloatCore + FromPrimitive> FromPrimitive for NotNan<T> {
    fn from_i64(n: i64) -> Option<Self> {
        T::from_i64(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_u64(n: u64) -> Option<Self> {
        T::from_u64(n).and_then(|n| NotNan::new(n).ok())
    }

    fn from_isize(n: isize) -> Option<Self> {
        T::from_isize(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_i8(n: i8) -> Option<Self> {
        T::from_i8(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_i16(n: i16) -> Option<Self> {
        T::from_i16(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_i32(n: i32) -> Option<Self> {
        T::from_i32(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_usize(n: usize) -> Option<Self> {
        T::from_usize(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_u8(n: u8) -> Option<Self> {
        T::from_u8(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_u16(n: u16) -> Option<Self> {
        T::from_u16(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_u32(n: u32) -> Option<Self> {
        T::from_u32(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_f32(n: f32) -> Option<Self> {
        T::from_f32(n).and_then(|n| NotNan::new(n).ok())
    }
    fn from_f64(n: f64) -> Option<Self> {
        T::from_f64(n).and_then(|n| NotNan::new(n).ok())
    }
}

impl<T: FloatCore> ToPrimitive for NotNan<T> {
    fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }
    fn to_u64(&self) -> Option<u64> {
        self.0.to_u64()
    }

    fn to_isize(&self) -> Option<isize> {
        self.0.to_isize()
    }
    fn to_i8(&self) -> Option<i8> {
        self.0.to_i8()
    }
    fn to_i16(&self) -> Option<i16> {
        self.0.to_i16()
    }
    fn to_i32(&self) -> Option<i32> {
        self.0.to_i32()
    }
    fn to_usize(&self) -> Option<usize> {
        self.0.to_usize()
    }
    fn to_u8(&self) -> Option<u8> {
        self.0.to_u8()
    }
    fn to_u16(&self) -> Option<u16> {
        self.0.to_u16()
    }
    fn to_u32(&self) -> Option<u32> {
        self.0.to_u32()
    }
    fn to_f32(&self) -> Option<f32> {
        self.0.to_f32()
    }
    fn to_f64(&self) -> Option<f64> {
        self.0.to_f64()
    }
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
            ParseNotNanError::ParseFloatError(e) => write!(f, "Parse error: {e}"),
            ParseNotNanError::IsNaN => write!(f, "NotNan parser encounter a NaN"),
        }
    }
}

impl<T: FloatCore> Num for NotNan<T> {
    type FromStrRadixErr = ParseNotNanError<T::FromStrRadixErr>;

    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        T::from_str_radix(src, radix)
            .map_err(ParseNotNanError::ParseFloatError)
            .and_then(|n| NotNan::new(n).map_err(|_| ParseNotNanError::IsNaN))
    }
}

impl<T: FloatCore + Signed> Signed for NotNan<T> {
    #[inline]
    fn abs(&self) -> Self {
        NotNan(self.0.abs())
    }

    fn abs_sub(&self, other: &Self) -> Self {
        NotNan::new(Signed::abs_sub(&self.0, &other.0)).expect("Subtraction resulted in NaN")
    }

    #[inline]
    fn signum(&self) -> Self {
        NotNan(self.0.signum())
    }
    #[inline]
    fn is_positive(&self) -> bool {
        self.0.is_positive()
    }
    #[inline]
    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

impl<T: FloatCore> NumCast for NotNan<T> {
    fn from<F: ToPrimitive>(n: F) -> Option<Self> {
        T::from(n).and_then(|n| NotNan::new(n).ok())
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
impl<T: Real + FloatCore> Real for NotNan<T> {
    fn min_value() -> Self {
        NotNan(<T as Real>::min_value())
    }
    fn min_positive_value() -> Self {
        NotNan(<T as Real>::min_positive_value())
    }
    fn epsilon() -> Self {
        NotNan(Real::epsilon())
    }
    fn max_value() -> Self {
        NotNan(<T as Real>::max_value())
    }
    fn floor(self) -> Self {
        NotNan(Real::floor(self.0))
    }
    fn ceil(self) -> Self {
        NotNan(Real::ceil(self.0))
    }
    fn round(self) -> Self {
        NotNan(Real::round(self.0))
    }
    fn trunc(self) -> Self {
        NotNan(Real::trunc(self.0))
    }
    fn fract(self) -> Self {
        NotNan(Real::fract(self.0))
    }
    fn abs(self) -> Self {
        NotNan(Real::abs(self.0))
    }
    fn signum(self) -> Self {
        NotNan(Real::signum(self.0))
    }
    fn is_sign_positive(self) -> bool {
        Real::is_sign_positive(self.0)
    }
    fn is_sign_negative(self) -> bool {
        Real::is_sign_negative(self.0)
    }
    fn mul_add(self, a: Self, b: Self) -> Self {
        NotNan(self.0.mul_add(a.0, b.0))
    }
    fn recip(self) -> Self {
        NotNan(Real::recip(self.0))
    }
    fn powi(self, n: i32) -> Self {
        NotNan(Real::powi(self.0, n))
    }
    fn powf(self, n: Self) -> Self {
        // Panics if  self < 0 and n is not an integer
        NotNan::new(self.0.powf(n.0)).expect("Power resulted in NaN")
    }
    fn sqrt(self) -> Self {
        // Panics if self < 0
        NotNan::new(self.0.sqrt()).expect("Square root resulted in NaN")
    }
    fn exp(self) -> Self {
        NotNan(self.0.exp())
    }
    fn exp2(self) -> Self {
        NotNan(self.0.exp2())
    }
    fn ln(self) -> Self {
        // Panics if self <= 0
        NotNan::new(self.0.ln()).expect("Natural logarithm resulted in NaN")
    }
    fn log(self, base: Self) -> Self {
        // Panics if self <= 0 or base <= 0
        NotNan::new(self.0.log(base.0)).expect("Logarithm resulted in NaN")
    }
    fn log2(self) -> Self {
        // Panics if self <= 0
        NotNan::new(self.0.log2()).expect("Logarithm resulted in NaN")
    }
    fn log10(self) -> Self {
        // Panics if self <= 0
        NotNan::new(self.0.log10()).expect("Logarithm resulted in NaN")
    }
    fn to_degrees(self) -> Self {
        NotNan(Real::to_degrees(self.0))
    }
    fn to_radians(self) -> Self {
        NotNan(Real::to_radians(self.0))
    }
    fn max(self, other: Self) -> Self {
        NotNan(Real::max(self.0, other.0))
    }
    fn min(self, other: Self) -> Self {
        NotNan(Real::min(self.0, other.0))
    }
    fn abs_sub(self, other: Self) -> Self {
        NotNan(self.0.abs_sub(other.0))
    }
    fn cbrt(self) -> Self {
        NotNan(self.0.cbrt())
    }
    fn hypot(self, other: Self) -> Self {
        NotNan(self.0.hypot(other.0))
    }
    fn sin(self) -> Self {
        // Panics if self is +/-infinity
        NotNan::new(self.0.sin()).expect("Sine resulted in NaN")
    }
    fn cos(self) -> Self {
        // Panics if self is +/-infinity
        NotNan::new(self.0.cos()).expect("Cosine resulted in NaN")
    }
    fn tan(self) -> Self {
        // Panics if self is +/-infinity or self == pi/2 + k*pi
        NotNan::new(self.0.tan()).expect("Tangent resulted in NaN")
    }
    fn asin(self) -> Self {
        // Panics if self < -1.0 or self > 1.0
        NotNan::new(self.0.asin()).expect("Arcsine resulted in NaN")
    }
    fn acos(self) -> Self {
        // Panics if self < -1.0 or self > 1.0
        NotNan::new(self.0.acos()).expect("Arccosine resulted in NaN")
    }
    fn atan(self) -> Self {
        NotNan(self.0.atan())
    }
    fn atan2(self, other: Self) -> Self {
        NotNan(self.0.atan2(other.0))
    }
    fn sin_cos(self) -> (Self, Self) {
        // Panics if self is +/-infinity
        let (a, b) = self.0.sin_cos();
        (
            NotNan::new(a).expect("Sine resulted in NaN"),
            NotNan::new(b).expect("Cosine resulted in NaN"),
        )
    }
    fn exp_m1(self) -> Self {
        NotNan(self.0.exp_m1())
    }
    fn ln_1p(self) -> Self {
        // Panics if self <= -1.0
        NotNan::new(self.0.ln_1p()).expect("Natural logarithm resulted in NaN")
    }
    fn sinh(self) -> Self {
        NotNan(self.0.sinh())
    }
    fn cosh(self) -> Self {
        NotNan(self.0.cosh())
    }
    fn tanh(self) -> Self {
        NotNan(self.0.tanh())
    }
    fn asinh(self) -> Self {
        NotNan(self.0.asinh())
    }
    fn acosh(self) -> Self {
        // Panics if self < 1.0
        NotNan::new(self.0.acosh()).expect("Arccosh resulted in NaN")
    }
    fn atanh(self) -> Self {
        // Panics if self < -1.0 or self > 1.0
        NotNan::new(self.0.atanh()).expect("Arctanh resulted in NaN")
    }
}

macro_rules! impl_float_const_method {
    ($wrapper:expr, $method:ident) => {
        #[allow(non_snake_case)]
        #[allow(clippy::redundant_closure_call)]
        fn $method() -> Self {
            $wrapper(T::$method())
        }
    };
}

macro_rules! impl_float_const {
    ($type:ident, $wrapper:expr) => {
        impl<T: FloatConst> FloatConst for $type<T> {
            impl_float_const_method!($wrapper, E);
            impl_float_const_method!($wrapper, FRAC_1_PI);
            impl_float_const_method!($wrapper, FRAC_1_SQRT_2);
            impl_float_const_method!($wrapper, FRAC_2_PI);
            impl_float_const_method!($wrapper, FRAC_2_SQRT_PI);
            impl_float_const_method!($wrapper, FRAC_PI_2);
            impl_float_const_method!($wrapper, FRAC_PI_3);
            impl_float_const_method!($wrapper, FRAC_PI_4);
            impl_float_const_method!($wrapper, FRAC_PI_6);
            impl_float_const_method!($wrapper, FRAC_PI_8);
            impl_float_const_method!($wrapper, LN_10);
            impl_float_const_method!($wrapper, LN_2);
            impl_float_const_method!($wrapper, LOG10_E);
            impl_float_const_method!($wrapper, LOG2_E);
            impl_float_const_method!($wrapper, PI);
            impl_float_const_method!($wrapper, SQRT_2);
        }
    };
}

impl_float_const!(OrderedFloat, OrderedFloat);
// Float constants are not NaN.
impl_float_const!(NotNan, |x| unsafe { NotNan::new_unchecked(x) });

mod hash_internals {
    pub trait SealedTrait: Copy + num_traits::float::FloatCore {
        type Bits: core::hash::Hash;

        const CANONICAL_NAN_BITS: Self::Bits;

        fn canonical_bits(self) -> Self::Bits;
    }

    impl SealedTrait for f32 {
        type Bits = u32;

        const CANONICAL_NAN_BITS: u32 = 0x7fc00000;

        fn canonical_bits(self) -> u32 {
            // -0.0 + 0.0 == +0.0 under IEEE754 roundTiesToEven rounding mode,
            // which Rust guarantees. Thus by adding a positive zero we
            // canonicalize signed zero without any branches in one instruction.
            (self + 0.0).to_bits()
        }
    }

    impl SealedTrait for f64 {
        type Bits = u64;

        const CANONICAL_NAN_BITS: u64 = 0x7ff8000000000000;

        fn canonical_bits(self) -> u64 {
            (self + 0.0).to_bits()
        }
    }
}

/// The built-in floating point types `f32` and `f64`.
///
/// This is a "sealed" trait that cannot be implemented for any other types.
pub trait PrimitiveFloat: hash_internals::SealedTrait {}
impl PrimitiveFloat for f32 {}
impl PrimitiveFloat for f64 {}

impl<T: PrimitiveFloat> Hash for OrderedFloat<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        let bits = if self.0.is_nan() {
            T::CANONICAL_NAN_BITS
        } else {
            self.0.canonical_bits()
        };
        bits.hash(hasher);
    }
}

impl<T: PrimitiveFloat> Hash for NotNan<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.canonical_bits().hash(hasher);
    }
}

#[cfg(feature = "serde")]
mod impl_serde {
    extern crate serde;
    use self::serde::de::{Error, Unexpected};
    use self::serde::{Deserialize, Deserializer, Serialize, Serializer};
    use super::{NotNan, OrderedFloat};
    use core::f64;
    use num_traits::float::FloatCore;

    #[cfg(test)]
    extern crate serde_test;
    #[cfg(test)]
    use self::serde_test::{assert_de_tokens_error, assert_tokens, Token};

    impl<T: FloatCore + Serialize> Serialize for OrderedFloat<T> {
        #[inline]
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            self.0.serialize(s)
        }
    }

    impl<'de, T: FloatCore + Deserialize<'de>> Deserialize<'de> for OrderedFloat<T> {
        #[inline]
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            T::deserialize(d).map(OrderedFloat)
        }
    }

    impl<T: FloatCore + Serialize> Serialize for NotNan<T> {
        #[inline]
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            self.0.serialize(s)
        }
    }

    impl<'de, T: FloatCore + Deserialize<'de>> Deserialize<'de> for NotNan<T> {
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
            "invalid value: floating point `NaN`, expected float (but not NaN)",
        );
    }
}

#[cfg(any(feature = "rkyv_16", feature = "rkyv_32", feature = "rkyv_64"))]
mod impl_rkyv {
    use super::{NotNan, OrderedFloat};
    use num_traits::float::FloatCore;
    #[cfg(test)]
    use rkyv::{archived_root, ser::Serializer};
    use rkyv::{Archive, Deserialize, Fallible, Serialize};

    #[cfg(test)]
    type DefaultSerializer = rkyv::ser::serializers::CoreSerializer<16, 16>;
    #[cfg(test)]
    type DefaultDeserializer = rkyv::Infallible;

    impl<T: FloatCore + Archive> Archive for OrderedFloat<T> {
        type Archived = OrderedFloat<T::Archived>;

        type Resolver = T::Resolver;

        unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
            self.0.resolve(pos, resolver, out.cast())
        }
    }

    impl<T: FloatCore + Serialize<S>, S: Fallible + ?Sized> Serialize<S> for OrderedFloat<T> {
        fn serialize(&self, s: &mut S) -> Result<Self::Resolver, S::Error> {
            self.0.serialize(s)
        }
    }

    impl<T: FloatCore, AT: Deserialize<T, D>, D: Fallible + ?Sized> Deserialize<OrderedFloat<T>, D>
        for OrderedFloat<AT>
    {
        fn deserialize(&self, d: &mut D) -> Result<OrderedFloat<T>, D::Error> {
            self.0.deserialize(d).map(OrderedFloat)
        }
    }

    impl<T: FloatCore + Archive> Archive for NotNan<T> {
        type Archived = NotNan<T::Archived>;

        type Resolver = T::Resolver;

        unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
            self.0.resolve(pos, resolver, out.cast())
        }
    }

    impl<T: FloatCore + Serialize<S>, S: Fallible + ?Sized> Serialize<S> for NotNan<T> {
        fn serialize(&self, s: &mut S) -> Result<Self::Resolver, S::Error> {
            self.0.serialize(s)
        }
    }

    impl<T: FloatCore, AT: Deserialize<T, D>, D: Fallible + ?Sized> Deserialize<NotNan<T>, D>
        for NotNan<AT>
    {
        fn deserialize(&self, d: &mut D) -> Result<NotNan<T>, D::Error> {
            self.0.deserialize(d).map(NotNan)
        }
    }

    macro_rules! rkyv_eq_ord {
        ($main:ident, $float:ty, $rend:ty) => {
            impl PartialEq<$main<$float>> for $main<$rend> {
                fn eq(&self, other: &$main<$float>) -> bool {
                    other.eq(&self.0.value())
                }
            }
            impl PartialEq<$main<$rend>> for $main<$float> {
                fn eq(&self, other: &$main<$rend>) -> bool {
                    self.eq(&other.0.value())
                }
            }

            impl PartialOrd<$main<$float>> for $main<$rend> {
                fn partial_cmp(&self, other: &$main<$float>) -> Option<core::cmp::Ordering> {
                    self.0.value().partial_cmp(other)
                }
            }

            impl PartialOrd<$main<$rend>> for $main<$float> {
                fn partial_cmp(&self, other: &$main<$rend>) -> Option<core::cmp::Ordering> {
                    other
                        .0
                        .value()
                        .partial_cmp(self)
                        .map(core::cmp::Ordering::reverse)
                }
            }
        };
    }

    rkyv_eq_ord! { OrderedFloat, f32, rkyv::rend::f32_le }
    rkyv_eq_ord! { OrderedFloat, f32, rkyv::rend::f32_be }
    rkyv_eq_ord! { OrderedFloat, f64, rkyv::rend::f64_le }
    rkyv_eq_ord! { OrderedFloat, f64, rkyv::rend::f64_be }
    rkyv_eq_ord! { NotNan, f32, rkyv::rend::f32_le }
    rkyv_eq_ord! { NotNan, f32, rkyv::rend::f32_be }
    rkyv_eq_ord! { NotNan, f64, rkyv::rend::f64_le }
    rkyv_eq_ord! { NotNan, f64, rkyv::rend::f64_be }

    #[cfg(feature = "rkyv_ck")]
    use super::FloatIsNan;
    #[cfg(feature = "rkyv_ck")]
    use core::convert::Infallible;
    #[cfg(feature = "rkyv_ck")]
    use rkyv::bytecheck::CheckBytes;

    #[cfg(feature = "rkyv_ck")]
    impl<C: ?Sized, T: FloatCore + CheckBytes<C>> CheckBytes<C> for OrderedFloat<T> {
        type Error = Infallible;

        #[inline]
        unsafe fn check_bytes<'a>(value: *const Self, _: &mut C) -> Result<&'a Self, Self::Error> {
            Ok(&*value)
        }
    }

    #[cfg(feature = "rkyv_ck")]
    impl<C: ?Sized, T: FloatCore + CheckBytes<C>> CheckBytes<C> for NotNan<T> {
        type Error = FloatIsNan;

        #[inline]
        unsafe fn check_bytes<'a>(value: *const Self, _: &mut C) -> Result<&'a Self, Self::Error> {
            Self::new(*(value as *const T)).map(|_| &*value)
        }
    }

    #[test]
    fn test_ordered_float() {
        let float = OrderedFloat(1.0f64);
        let mut serializer = DefaultSerializer::default();
        serializer
            .serialize_value(&float)
            .expect("failed to archive value");
        let len = serializer.pos();
        let buffer = serializer.into_serializer().into_inner();

        let archived_value = unsafe { archived_root::<OrderedFloat<f64>>(&buffer[0..len]) };
        assert_eq!(archived_value, &float);
        let mut deserializer = DefaultDeserializer::default();
        let deser_float: OrderedFloat<f64> = archived_value.deserialize(&mut deserializer).unwrap();
        assert_eq!(deser_float, float);
    }

    #[test]
    fn test_not_nan() {
        let float = NotNan(1.0f64);
        let mut serializer = DefaultSerializer::default();
        serializer
            .serialize_value(&float)
            .expect("failed to archive value");
        let len = serializer.pos();
        let buffer = serializer.into_serializer().into_inner();

        let archived_value = unsafe { archived_root::<NotNan<f64>>(&buffer[0..len]) };
        assert_eq!(archived_value, &float);
        let mut deserializer = DefaultDeserializer::default();
        let deser_float: NotNan<f64> = archived_value.deserialize(&mut deserializer).unwrap();
        assert_eq!(deser_float, float);
    }
}

#[cfg(feature = "speedy")]
mod impl_speedy {
    use super::{NotNan, OrderedFloat};
    use num_traits::float::FloatCore;
    use speedy::{Context, Readable, Reader, Writable, Writer};

    impl<C, T> Writable<C> for OrderedFloat<T>
    where
        C: Context,
        T: Writable<C>,
    {
        fn write_to<W: ?Sized + Writer<C>>(&self, writer: &mut W) -> Result<(), C::Error> {
            self.0.write_to(writer)
        }

        fn bytes_needed(&self) -> Result<usize, C::Error> {
            self.0.bytes_needed()
        }
    }

    impl<C, T> Writable<C> for NotNan<T>
    where
        C: Context,
        T: Writable<C>,
    {
        fn write_to<W: ?Sized + Writer<C>>(&self, writer: &mut W) -> Result<(), C::Error> {
            self.0.write_to(writer)
        }

        fn bytes_needed(&self) -> Result<usize, C::Error> {
            self.0.bytes_needed()
        }
    }

    impl<'a, T, C: Context> Readable<'a, C> for OrderedFloat<T>
    where
        T: Readable<'a, C>,
    {
        fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
            T::read_from(reader).map(OrderedFloat)
        }

        fn minimum_bytes_needed() -> usize {
            T::minimum_bytes_needed()
        }
    }

    impl<'a, T: FloatCore, C: Context> Readable<'a, C> for NotNan<T>
    where
        T: Readable<'a, C>,
    {
        fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
            let value: T = reader.read_value()?;
            Self::new(value).map_err(|error| {
                speedy::Error::custom(std::format!("failed to read NotNan: {error}")).into()
            })
        }

        fn minimum_bytes_needed() -> usize {
            T::minimum_bytes_needed()
        }
    }

    #[test]
    fn test_ordered_float() {
        let float = OrderedFloat(1.0f64);
        let buffer = float.write_to_vec().unwrap();
        let deser_float: OrderedFloat<f64> = OrderedFloat::read_from_buffer(&buffer).unwrap();
        assert_eq!(deser_float, float);
    }

    #[test]
    fn test_not_nan() {
        let float = NotNan(1.0f64);
        let buffer = float.write_to_vec().unwrap();
        let deser_float: NotNan<f64> = NotNan::read_from_buffer(&buffer).unwrap();
        assert_eq!(deser_float, float);
    }

    #[test]
    fn test_not_nan_with_nan() {
        let nan_buf = f64::nan().write_to_vec().unwrap();
        let nan_err: Result<NotNan<f64>, _> = NotNan::read_from_buffer(&nan_buf);
        assert!(nan_err.is_err());
    }
}

#[cfg(feature = "borsh")]
mod impl_borsh {
    extern crate borsh;
    use super::{NotNan, OrderedFloat};
    use num_traits::float::FloatCore;

    impl<T> borsh::BorshSerialize for OrderedFloat<T>
    where
        T: borsh::BorshSerialize,
    {
        #[inline]
        fn serialize<W: borsh::io::Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
            <T as borsh::BorshSerialize>::serialize(&self.0, writer)
        }
    }

    impl<T> borsh::BorshDeserialize for OrderedFloat<T>
    where
        T: borsh::BorshDeserialize,
    {
        #[inline]
        fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
            <T as borsh::BorshDeserialize>::deserialize_reader(reader).map(Self)
        }
    }

    impl<T> borsh::BorshSerialize for NotNan<T>
    where
        T: borsh::BorshSerialize,
    {
        #[inline]
        fn serialize<W: borsh::io::Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
            <T as borsh::BorshSerialize>::serialize(&self.0, writer)
        }
    }

    impl<T> borsh::BorshDeserialize for NotNan<T>
    where
        T: FloatCore + borsh::BorshDeserialize,
    {
        #[inline]
        fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
            let float = <T as borsh::BorshDeserialize>::deserialize_reader(reader)?;
            NotNan::new(float).map_err(|_| {
                borsh::io::Error::new(
                    borsh::io::ErrorKind::InvalidData,
                    "expected a non-NaN float",
                )
            })
        }
    }

    #[test]
    fn test_ordered_float() {
        let float = crate::OrderedFloat(1.0f64);
        let buffer = borsh::to_vec(&float).expect("failed to serialize value");
        let deser_float: crate::OrderedFloat<f64> =
            borsh::from_slice(&buffer).expect("failed to deserialize value");
        assert_eq!(deser_float, float);
    }

    #[test]
    fn test_not_nan() {
        let float = crate::NotNan(1.0f64);
        let buffer = borsh::to_vec(&float).expect("failed to serialize value");
        let deser_float: crate::NotNan<f64> =
            borsh::from_slice(&buffer).expect("failed to deserialize value");
        assert_eq!(deser_float, float);
    }
}

#[cfg(all(feature = "std", feature = "schemars"))]
mod impl_schemars {
    extern crate schemars;
    use self::schemars::generate::SchemaGenerator;
    use self::schemars::Schema;
    use super::{NotNan, OrderedFloat};

    macro_rules! primitive_float_impl {
        ($type:ty, $schema_name:literal) => {
            impl schemars::JsonSchema for $type {
                fn inline_schema() -> bool {
                    true
                }

                fn schema_name() -> std::borrow::Cow<'static, str> {
                    std::borrow::Cow::from($schema_name)
                }

                fn json_schema(_: &mut SchemaGenerator) -> Schema {
                    schemars::json_schema!({
                        "type": "number",
                        "title": $schema_name,
                        "format": $schema_name,
                    })
                }
            }
        };
    }

    primitive_float_impl!(OrderedFloat<f32>, "float");
    primitive_float_impl!(OrderedFloat<f64>, "double");
    primitive_float_impl!(NotNan<f32>, "float");
    primitive_float_impl!(NotNan<f64>, "double");

    #[test]
    fn schema_generation_does_not_panic_for_common_floats() {
        fn test<T: schemars::JsonSchema>(title: &str) {
            let schema = schemars::generate::SchemaGenerator::default().into_root_schema_for::<T>();

            assert_eq!(schema.get("type").unwrap().as_str().unwrap(), "number");
            assert_eq!(schema.get("title").unwrap().as_str().unwrap(), title);
            assert_eq!(schema.get("format").unwrap().as_str().unwrap(), title);
        }

        test::<OrderedFloat<f32>>("float");
        test::<NotNan<f32>>("float");
        test::<OrderedFloat<f64>>("double");
        test::<NotNan<f64>>("double");
    }

    #[test]
    fn ordered_float_schema_match_primitive_schema() {
        fn test<Wrapped: schemars::JsonSchema, Inner: schemars::JsonSchema>() {
            let of_schema =
                schemars::generate::SchemaGenerator::default().into_root_schema_for::<Wrapped>();
            let prim_schema =
                schemars::generate::SchemaGenerator::default().into_root_schema_for::<Inner>();
            assert_eq!(of_schema, prim_schema);
        }

        test::<OrderedFloat<f32>, f32>();
        test::<NotNan<f32>, f32>();
        test::<OrderedFloat<f64>, f64>();
        test::<NotNan<f64>, f64>();
    }
}

#[cfg(feature = "rand")]
mod impl_rand {
    use super::{NotNan, OrderedFloat};
    use rand::distributions::uniform::*;
    use rand::distributions::{Distribution, Open01, OpenClosed01, Standard};
    use rand::Rng;

    macro_rules! impl_distribution {
        ($dist:ident, $($f:ty),+) => {
            $(
            impl Distribution<NotNan<$f>> for $dist {
                fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> NotNan<$f> {
                    // 'rand' never generates NaN values in the Standard, Open01, or
                    // OpenClosed01 distributions. Using 'new_unchecked' is therefore
                    // safe.
                    unsafe { NotNan::new_unchecked(self.sample(rng)) }
                }
            }

            impl Distribution<OrderedFloat<$f>> for $dist {
                fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OrderedFloat<$f> {
                    OrderedFloat(self.sample(rng))
                }
            }
            )*
        }
    }

    impl_distribution! { Standard, f32, f64 }
    impl_distribution! { Open01, f32, f64 }
    impl_distribution! { OpenClosed01, f32, f64 }

    /// A sampler for a uniform distribution
    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct UniformNotNan<T>(UniformFloat<T>);
    impl SampleUniform for NotNan<f32> {
        type Sampler = UniformNotNan<f32>;
    }
    impl SampleUniform for NotNan<f64> {
        type Sampler = UniformNotNan<f64>;
    }
    impl<T> PartialEq for UniformNotNan<T>
    where
        UniformFloat<T>: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    /// A sampler for a uniform distribution
    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct UniformOrdered<T>(UniformFloat<T>);
    impl SampleUniform for OrderedFloat<f32> {
        type Sampler = UniformOrdered<f32>;
    }
    impl SampleUniform for OrderedFloat<f64> {
        type Sampler = UniformOrdered<f64>;
    }
    impl<T> PartialEq for UniformOrdered<T>
    where
        UniformFloat<T>: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    macro_rules! impl_uniform_sampler {
        ($f:ty) => {
            impl UniformSampler for UniformNotNan<$f> {
                type X = NotNan<$f>;
                fn new<B1, B2>(low: B1, high: B2) -> Self
                where
                    B1: SampleBorrow<Self::X> + Sized,
                    B2: SampleBorrow<Self::X> + Sized,
                {
                    UniformNotNan(UniformFloat::<$f>::new(low.borrow().0, high.borrow().0))
                }
                fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
                where
                    B1: SampleBorrow<Self::X> + Sized,
                    B2: SampleBorrow<Self::X> + Sized,
                {
                    UniformSampler::new(low, high)
                }
                fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
                    // UniformFloat.sample() will never return NaN.
                    unsafe { NotNan::new_unchecked(self.0.sample(rng)) }
                }
            }

            impl UniformSampler for UniformOrdered<$f> {
                type X = OrderedFloat<$f>;
                fn new<B1, B2>(low: B1, high: B2) -> Self
                where
                    B1: SampleBorrow<Self::X> + Sized,
                    B2: SampleBorrow<Self::X> + Sized,
                {
                    UniformOrdered(UniformFloat::<$f>::new(low.borrow().0, high.borrow().0))
                }
                fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
                where
                    B1: SampleBorrow<Self::X> + Sized,
                    B2: SampleBorrow<Self::X> + Sized,
                {
                    UniformSampler::new(low, high)
                }
                fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
                    OrderedFloat(self.0.sample(rng))
                }
            }
        };
    }

    impl_uniform_sampler! { f32 }
    impl_uniform_sampler! { f64 }

    #[cfg(all(test, feature = "randtest"))]
    mod tests {
        use super::*;

        fn sample_fuzz<T>()
        where
            Standard: Distribution<NotNan<T>>,
            Open01: Distribution<NotNan<T>>,
            OpenClosed01: Distribution<NotNan<T>>,
            Standard: Distribution<OrderedFloat<T>>,
            Open01: Distribution<OrderedFloat<T>>,
            OpenClosed01: Distribution<OrderedFloat<T>>,
            T: crate::Float,
        {
            let mut rng = rand::thread_rng();
            let f1: NotNan<T> = rng.sample(Standard);
            let f2: NotNan<T> = rng.sample(Open01);
            let f3: NotNan<T> = rng.sample(OpenClosed01);
            let _: OrderedFloat<T> = rng.sample(Standard);
            let _: OrderedFloat<T> = rng.sample(Open01);
            let _: OrderedFloat<T> = rng.sample(OpenClosed01);
            assert!(!f1.into_inner().is_nan());
            assert!(!f2.into_inner().is_nan());
            assert!(!f3.into_inner().is_nan());
        }

        #[test]
        fn sampling_f32_does_not_panic() {
            sample_fuzz::<f32>();
        }

        #[test]
        fn sampling_f64_does_not_panic() {
            sample_fuzz::<f64>();
        }

        #[test]
        #[should_panic]
        fn uniform_sampling_panic_on_infinity_notnan() {
            let (low, high) = (
                NotNan::new(0f64).unwrap(),
                NotNan::new(f64::INFINITY).unwrap(),
            );
            let uniform = Uniform::new(low, high);
            let _ = uniform.sample(&mut rand::thread_rng());
        }

        #[test]
        #[should_panic]
        fn uniform_sampling_panic_on_infinity_ordered() {
            let (low, high) = (OrderedFloat(0f64), OrderedFloat(f64::INFINITY));
            let uniform = Uniform::new(low, high);
            let _ = uniform.sample(&mut rand::thread_rng());
        }

        #[test]
        #[should_panic]
        fn uniform_sampling_panic_on_nan_ordered() {
            let (low, high) = (OrderedFloat(0f64), OrderedFloat(f64::NAN));
            let uniform = Uniform::new(low, high);
            let _ = uniform.sample(&mut rand::thread_rng());
        }
    }
}

#[cfg(feature = "proptest")]
mod impl_proptest {
    use super::{NotNan, OrderedFloat};
    use proptest::arbitrary::{Arbitrary, StrategyFor};
    use proptest::num::{f32, f64};
    use proptest::strategy::{FilterMap, Map, Strategy};
    use std::convert::TryFrom;

    macro_rules! impl_arbitrary {
        ($($f:ident),+) => {
            $(
                impl Arbitrary for NotNan<$f> {
                    type Strategy = FilterMap<StrategyFor<$f>, fn(_: $f) -> Option<NotNan<$f>>>;
                    type Parameters = <$f as Arbitrary>::Parameters;
                    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
                        <$f>::arbitrary_with(params)
                            .prop_filter_map("filter nan values", |f| NotNan::try_from(f).ok())
                    }
                }

                impl Arbitrary for OrderedFloat<$f> {
                    type Strategy = Map<StrategyFor<$f>, fn(_: $f) -> OrderedFloat<$f>>;
                    type Parameters = <$f as Arbitrary>::Parameters;
                    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
                        <$f>::arbitrary_with(params).prop_map(|f| OrderedFloat::from(f))
                    }
                }
            )*
        }
    }
    impl_arbitrary! { f32, f64 }
}

#[cfg(feature = "arbitrary")]
mod impl_arbitrary {
    use super::{FloatIsNan, NotNan, OrderedFloat};
    use arbitrary::{Arbitrary, Unstructured};
    use num_traits::FromPrimitive;

    macro_rules! impl_arbitrary {
        ($($f:ident),+) => {
            $(
                impl<'a> Arbitrary<'a> for NotNan<$f> {
                    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                        let float: $f = u.arbitrary()?;
                        match NotNan::new(float) {
                            Ok(notnan_value) => Ok(notnan_value),
                            Err(FloatIsNan) => {
                                // If our arbitrary float input was a NaN (encoded by exponent = max
                                // value), then replace it with a finite float, reusing the mantissa
                                // bits.
                                //
                                // This means the output is not uniformly distributed among all
                                // possible float values, but Arbitrary makes no promise that that
                                // is true.
                                //
                                // An alternative implementation would be to return an
                                // `arbitrary::Error`, but that is not as useful since it forces the
                                // caller to retry with new random/fuzzed data; and the precendent of
                                // `arbitrary`'s built-in implementations is to prefer the approach of
                                // mangling the input bits to fit.

                                let (mantissa, _exponent, sign) =
                                    num_traits::float::FloatCore::integer_decode(float);
                                let revised_float = <$f>::from_i64(
                                    i64::from(sign) * mantissa as i64
                                ).unwrap();

                                // If this unwrap() fails, then there is a bug in the above code.
                                Ok(NotNan::new(revised_float).unwrap())
                            }
                        }
                    }

                    fn size_hint(depth: usize) -> (usize, Option<usize>) {
                        <$f as Arbitrary>::size_hint(depth)
                    }
                }

                impl<'a> Arbitrary<'a> for OrderedFloat<$f> {
                    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                        let float: $f = u.arbitrary()?;
                        Ok(OrderedFloat::from(float))
                    }

                    fn size_hint(depth: usize) -> (usize, Option<usize>) {
                        <$f as Arbitrary>::size_hint(depth)
                    }
                }
            )*
        }
    }
    impl_arbitrary! { f32, f64 }
}

#[cfg(feature = "bytemuck")]
mod impl_bytemuck {
    use super::{FloatCore, NotNan, OrderedFloat};
    use bytemuck::{AnyBitPattern, CheckedBitPattern, NoUninit, Pod, TransparentWrapper, Zeroable};

    unsafe impl<T: Zeroable> Zeroable for OrderedFloat<T> {}

    // The zero bit pattern is indeed not a NaN bit pattern.
    unsafe impl<T: Zeroable> Zeroable for NotNan<T> {}

    unsafe impl<T: Pod> Pod for OrderedFloat<T> {}

    // `NotNan<T>` can only implement `NoUninit` and not `Pod`, since not every bit pattern is
    // valid (NaN bit patterns are invalid). `NoUninit` guarantees that we can read any bit pattern
    // from the value, which is fine in this case.
    unsafe impl<T: NoUninit> NoUninit for NotNan<T> {}

    unsafe impl<T: FloatCore + AnyBitPattern> CheckedBitPattern for NotNan<T> {
        type Bits = T;

        fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
            !bits.is_nan()
        }
    }

    // OrderedFloat allows any value of the contained type, so it is a TransparentWrapper.
    // NotNan does not, so it is not.
    unsafe impl<T> TransparentWrapper<T> for OrderedFloat<T> {}

    #[test]
    fn test_not_nan_bit_pattern() {
        use bytemuck::checked::{try_cast, CheckedCastError};

        let nan = f64::NAN;
        assert_eq!(
            try_cast::<f64, NotNan<f64>>(nan),
            Err(CheckedCastError::InvalidBitPattern),
        );

        let pi = core::f64::consts::PI;
        assert!(try_cast::<f64, NotNan<f64>>(pi).is_ok());
    }
}
