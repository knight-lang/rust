use crate::{Value, Boolean, Text};
use crate::value::{SHIFT, Tag, ValueKind, Runnable};
use std::fmt::{self, Display, Formatter};
// use std::convert::TryFrom;
use try_traits::ops::{TryAdd, TrySub, TryMul, TryDiv, TryRem, TryNeg};

/// The number type that's used internally within [`Number`].
pub type NumberInner = i64;

/// The numeric type in Knight.
///
/// This crate actually uses "`i61`"s as numbers, and so this type represents that. Internally, it's represented as a
/// [`NumberInner`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(NumberInner);

// ensure the min/max values are valid.
const_assert_eq!((Number::MIN.get() << SHIFT) >> SHIFT, Number::MIN.get());
const_assert_ne!(((Number::MIN.get() - 1) << SHIFT) >> SHIFT, Number::MIN.get() - 1);
const_assert_eq!((Number::MAX.get() << SHIFT) >> SHIFT, Number::MAX.get());
const_assert_ne!(((Number::MAX.get() + 1) << SHIFT) >> SHIFT, Number::MAX.get() + 1);

impl Number {
	/// A constant representing the value zero.
	pub const ZERO: Self = Self(0);

	/// A constant representing the value One.
	pub const ONE: Self = Self(1);

	/// The maximum value a [`Number`] can contain.
	pub const MAX: Self = Self(NumberInner::MAX >> SHIFT);

	/// The minimum value a [`Number`] can contain.
	pub const MIN: Self = Self(!Self::MAX.0);

	/// Try to create a new [`Number`], returning [`None`] if the number is out of bounds.
	pub const fn new(num: i64) -> Option<Self> {
		if Self::MIN.0 <= num && num <= Self::MAX.0 {
			Some(Self(num))
		} else {
			None
		}
	}

	/// Creates a new [`Number`] **without** verifying that it's within bounds.
	///
	/// # Safety
	/// The passed number must be within the range `Number::MIN.get()..=Number::MAX.get()`.
	pub const unsafe fn new_unchecked(num: i64) -> Self {
		debug_assert_const!(Number::new(num).is_some());

		Self(num)
	}

	/// Creates a new [`Number`], truncating bits that are out of bounds.
	pub const fn new_truncate(num: i64) -> Self {
		unsafe {
			Self::new_unchecked((num << SHIFT) >> SHIFT)
		}
	}

	/// Fetches the [`NumberInner`] that `self` wraps.
	pub const fn get(self) -> NumberInner {
		self.0
	}
}

impl Display for Number {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.get(), f)
	}
}

impl From<Number> for Value<'_> {
	#[inline]
	fn from(num: Number) -> Self {
		unsafe {
			Self::new_tagged((num.get() as u64) << SHIFT, Tag::Number)
		}
	}
}

unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Number {
	type Ref = Self;

	fn is_value_a(value: &Value<'env>) -> bool {
		value.tag() == Tag::Number
	}

	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		Self::new_unchecked((value.raw() as NumberInner) >> SHIFT)
	}
}

impl<'env> Runnable<'env> for Number {
	#[inline]
	fn run(&self, _: &'env  crate::Environment) -> crate::Result<Value<'env>> {
		Ok((*self).into())
	}
}

impl From<Number> for Boolean {
	#[inline]
	fn from(number: Number) -> Self {
		number.get() != 0
	}
}

impl From<Number> for Text {
	#[inline]
	fn from(number: Number) -> Self {
		unsafe {
			Self::new_unchecked(number.to_string().into())
		}
	}
}


impl From<Number> for NumberInner {
	#[inline]
	fn from(number: Number) -> Self {
		number.get()
	}
}
// derive
// pub struct NumberTooLarge;

// impl TryFrom<NumberInner> for Number {
// 	type Error = std::option::NoneError;
// 	fn try_from(val: NumberInner) -> Result<Self, Self::Error> {
// 		Self::new(val)
// 	}
// }

#[derive(Debug)]
pub enum MathError {

}

impl TryAdd for Number {

}
