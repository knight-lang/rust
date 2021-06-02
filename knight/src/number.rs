//! Types relating to [`Number`]s.

use crate::{Value, Boolean, Text};
use crate::value::{SHIFT, Tag, ValueKind};
use crate::ops::{Runnable, ToText, TryAdd, TrySub, TryMul, TryDiv, TryRem, TryPow, TryNeg, Infallible};
use std::fmt::{self, Display, Formatter};
use std::num::TryFromIntError;
use std::convert::TryFrom;

/// The numeric type in Knight.
///
/// Because this crate uses tagged pointers to represent [`Value`](crate::Value)s, we're unable to use the full range of
/// an `i64`. As such, this type is used to represents numeric types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(i64);

impl Number {
	/// A constant representing the value zero.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(Number::ZERO.get(), 0);
	/// assert_eq!(Number::ZERO, Number::new(0).unwrap());
	/// ```
	pub const ZERO: Self = Self(0);

	/// A constant representing the value one.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(Number::ONE.get(), 1);
	/// assert_eq!(Number::ONE, Number::new(1).unwrap());
	/// ```
	pub const ONE: Self = Self(1);

	/// A constant representing the value negative one.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(Number::NEG_ONE.get(), -1);
	/// assert_eq!(Number::NEG_ONE, Number::new(-1).unwrap());
	/// ```
	pub const NEG_ONE: Self = Self(-1);

	/// The maximum value a [`Number`] can contain.
	///
	/// Note that when the `strict-numbers` feature is enabled, this is simply [`i32::MAX`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(Number::new(Number::MAX.get() - 1).unwrap().get(), Number::MAX.get() - 1);
	/// assert_eq!(Number::new(Number::MAX.get()), Some(Number::MAX));
	/// assert_eq!(Number::new(Number::MAX.get() + 1), None);
	/// ```
	pub const MAX: Self = Self(
		#[cfg(feature="strict-numbers")]
		{ i32::MAX as i64 },

		#[cfg(not(feature="strict-numbers"))]
		{ i64::MAX >> SHIFT }
	);

	/// The minimum value a [`Number`] can contain.
	///
	/// Note that when the `strict-numbers` feature is enabled, this is simply [`i32::MIN`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(Number::new(Number::MIN.get() + 1).unwrap().get(), Number::MIN.get() + 1);
	/// assert_eq!(Number::new(Number::MIN.get()), Some(Number::MIN));
	/// assert_eq!(Number::new(Number::MIN.get() - 1), None);
	/// ```
	pub const MIN: Self = Self(
		#[cfg(feature="strict-numbers")]
		{ i32::MIN as i64 },

		#[cfg(not(feature="strict-numbers"))]
		{ !Self::MAX.0 }
	);

	/// Try to create a new [`Number`], returning [`None`] if the number is out of bounds.
	///
	/// More specifically, this will return [`None`] if [`Number::MIN`] is larger than `num` or [`Number::MAX`] is
	/// smaller.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(Number::new(123).map(Number::get), Some(123));
	/// assert_eq!(Number::new(-456).map(Number::get), Some(-456));
	/// assert_eq!(Number::new(0), Some(Number::ZERO));
	///
	/// assert_eq!(Number::new(i64::MAX), None);
	/// assert_eq!(Number::new(i64::MIN), None);
	/// ```
	pub const fn new(num: i64) -> Option<Self> {
		// can't use a range, as that's not const.
		if Self::MIN.0 <= num && num <= Self::MAX.0 {
			Some(Self(num))
		} else {
			None
		}
	}

	/// Creates a new [`Number`] **without** verifying that it's within bounds.
	///
	/// # Safety
	/// The caller must ensure that `num` is within bounds---ie within the range `[`[`Number::MIN`], [`Number::MAX`]`]`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(unsafe { Number::new_unchecked(123) }.get(), 123);
	/// assert_eq!(unsafe { Number::new_unchecked(-456) }.get(), -456);
	/// ```
	#[inline]
	pub const unsafe fn new_unchecked(num: i64) -> Self {
		debug_assert_const!(Number::new(num).is_some());

		Self(num)
	}

	/// Creates a new [`Number`], truncating bits that are out of bounds.
	///
	/// For values within range (ie within `Number::MIN..=Number::MAX`), this simply returns the original number.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// // numbers within range simply return themselves.
	/// assert_eq!(Number::new_truncate(12345).get(), 12345);
	/// assert_eq!(Number::new_truncate(-678).get(), -678);
	///
	/// // numbers outside of range simply lop off the top few bits.
	/// assert_eq!(Number::new_truncate(Number::MIN.get() - 1).get(), !Number::MIN.get());
	/// assert_eq!(Number::new_truncate(Number::MAX.get() + 1).get(), !Number::MAX.get());
	/// ```
	#[inline]
	pub const fn new_truncate(num: i64) -> Self {
		#[cfg(feature = "strict-numbers")]
		{ Self(num as i32 as i64) }

		#[cfg(not(feature = "strict-numbers"))]
		{ Self((num << SHIFT) >> SHIFT) }
	}

	/// Fetches the number that `self` internally stores.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(Number::new(1234).unwrap().get(), 1234);
	/// assert_eq!(Number::new(-56789).unwrap().get(), -56789);
	/// ```
	#[inline]
	pub const fn get(self) -> i64 {
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
		// SAFETY:
		// - We shifted the number left, so we know the bottom `SHIFT` bits aren't set.
		// - All numbers are valid discriminant for `Tag::Number`
		unsafe {
			Self::new_tagged((num.get() as u64) << SHIFT, Tag::Number)
		}
	}
}

// SAFETY: 
// - `is_value_a` only returns true when we're made with a `Tag::Number`. Assuming all other `ValueKind`s are
//   well-defined, then it will only ever return `true` when the value was constructed via `Number::into`
// - `downcast_unchecked`, when passed a valid `Number` value, will always recover the original one.
unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Number {
	type Ref = Self;

	#[inline]
	fn is_value_a(value: &Value<'env>) -> bool {
		value.tag() == Tag::Number
	}

	#[inline]
	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value), "Number::downcast_unchecked ran with a bad value: {:#016x}", value.raw());

		// SAFETY: The caller guarantees that this function was only called with a valid `Number` value, so it
		// must be within range. (Additionally, shifting right by `SHIFT` makes it valid tautologically)
		Self::new_unchecked((value.raw() as i64) >> SHIFT)
	}
}

impl<'env> Runnable<'env> for Number {
	/// Runs the [`Number`] by simply converting it to a [`Value`].
	#[inline]
	fn run(&self, _: &'env crate::Environment) -> crate::Result<Value<'env>> {
		Ok((*self).into())
	}
}

impl From<Number> for Boolean {
	/// Returns `false` for [`Number::ZERO`] and `true` for all other values.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(false, Number::ZERO.into());
	/// assert_eq!(true, Number::ONE.into());
	/// assert_eq!(true, Number::new(123).unwrap().into());
	/// assert_eq!(true, Number::new(-14).unwrap().into());
	/// ```
	#[inline]
	fn from(number: Number) -> Self {
		number.get() != 0
	}
}

impl ToText<'_> for Number {
	type Error = Infallible;
	type Output = Text;

	#[inline]
	fn to_text(&self) -> Result<Self::Output, Self::Error> {
		// TODO: use some form of caching here.
		Ok(Text::new(self.to_string().into()).unwrap())
	}
}

macro_rules! impl_number_conversions {
	($($smaller:ty),*; $($larger:ty),*) => {
		$(
			impl From<$smaller> for Number {
				fn from(num: $smaller) -> Self {
					// SAFETY: `num` is always within range.
					unsafe {
						Self::new_unchecked(num as i64)
					}
				}
			}

			impl TryFrom<Number> for $smaller {
				type Error = TryFromIntError;

				fn try_from(num: Number) -> Result<Self, TryFromIntError> {
					Self::try_from(num.get())
				}
			}
		)*

		$(
			impl TryFrom<$larger> for Number {
				type Error = TryFromIntError;

				fn try_from(num: $larger) -> Result<Self, TryFromIntError> {
					i64::try_from(num)
						.ok()
						.and_then(Number::new)
						.ok_or_else(||
							match i8::try_from(-1234i32) {
								Ok(_) => unsafe { std::hint::unreachable_unchecked() },
								Err(err) => err
							}
						)
				}
			}

			impl From<Number> for $larger {
				fn from(num: Number) -> Self {
					num.get() as Self
				}
			}
		)*
	};
}

impl_number_conversions!(i8, u8, i16, u16, i32, u32; i64, u64, i128, u128);


/// Errors that can arise during math operations on a [`Number`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathError {
	/// An operation overflowed the allowed bounds for a [`Number`].
	///
	/// Note that this is only ever returned when the `checked-overflow` feature is enabled.
	Overflow,

	/// A division by zero was attempted.
	///
	/// Note that this isn't just division, but also includes modulo by zero and exponentiating zero by a negative power.
	DivisionByZero,

	/// Modulo was attempted with either a negative number or base.
	NegativeModulo
}

impl std::error::Error for MathError {}
impl Display for MathError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Overflow => write!(f, "an operation overflowed"),
			Self::DivisionByZero => write!(f, "division by zero attempted"),
			Self::NegativeModulo => write!(f, "modulo with negative numbers attempted"),
		}
	}
}

impl TryNeg for Number {
	type Error = MathError;
	type Output = Self;

	/// Attempts to negate `self`.
	///
	/// # Errors
	/// Without the `checked-overflow` feature, this will [truncate](Number:new_truncate) the result. With the feature, a
	/// [`MathError::Overflow`] will be returned if the result is unable to fit within a [`Number`]. (This is only
	/// possible when negating [`Number::MIN`].)
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Number, ops::TryNeg};
	/// let number = Number::new(123).unwrap();
	/// assert_eq!(number.try_neg().unwrap().get(), -123);
	/// ```
	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_neg(self) -> Result<Self::Output, Self::Error> {
		let number = self.get();

		cfg_if! {
			if #[cfg(feature="checked-overflow")] {
				number.checked_neg()
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(number.wrapping_neg()))
			}
		}
	}
}

impl TryAdd for Number {
	type Error = MathError;
	type Output = Self;

	/// Attempts to add `addend` to `self`, returning the sum.
	///
	/// # Errors
	/// Without the `checked-overflow` feature, this will [truncate](Number:new_truncate) the result. With the feature, a
	/// [`MathError::Overflow`] will be returned if the result is too large to fit within a [`Number`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Number, ops::TryAdd};
	/// let augend = Number::new(123).unwrap();
	/// let addend = Number::new(-12).unwrap();
	/// assert_eq!(augend.try_add(addend).unwrap().get(), 111);
	/// ```
	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_add(self, addend: Self) -> Result<Self::Output, Self::Error> {
		let augend = self.get();
		let addend = addend.get();

		cfg_if! {
			if #[cfg(feature="checked-overflow")] {
				augend.checked_add(addend)
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(augend.wrapping_add(addend)))
			}
		}
	}
}

impl TrySub for Number {
	type Error = MathError;
	type Output = Self;

	/// Attempts to subtract `subtrahend` from `self`, returning the difference.
	///
	/// # Errors
	/// Without the `checked-overflow` feature, this will [truncate](Number:new_truncate) the result. With the feature, a
	/// [`MathError::Overflow`] will be returned if the result is too small to fit within a [`Number`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Number, ops::TrySub};
	/// let minuend = Number::new(123).unwrap();
	/// let subtrahend = Number::new(-12).unwrap();
	/// assert_eq!(minuend.try_sub(subtrahend).unwrap().get(), 135);
	/// ```
	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_sub(self, subtrahend: Self) -> Result<Self::Output, Self::Error> {
		let minuend = self.get();
		let subtrahend = subtrahend.get();

		cfg_if! {
			if #[cfg(feature="checked-overflow")] {
				minuend.checked_sub(subtrahend)
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(minuend.wrapping_sub(subtrahend)))
			}
		}
	}
}

impl TryMul for Number {
	type Error = MathError;
	type Output = Self;

	/// Attempts to multiply `self` by `multiplier`, returning their product.
	///
	/// # Errors
	/// Without the `checked-overflow` feature, this will [truncate](Number:new_truncate) the result. With the feature, a
	/// [`MathError::Overflow`] will be returned if the result is unable to fit within a [`Number`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Number, ops::TryMul};
	/// let multiplicand = Number::new(123).unwrap();
	/// let multiplier = Number::new(-12).unwrap();
	/// assert_eq!(multiplicand.try_mul(multiplier).unwrap().get(), -1476);
	/// ```
	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_mul(self, multiplier: Self) -> Result<Self::Output, Self::Error> {
		let multiplicand = self.get();
		let multiplier = multiplier.get();

		cfg_if! {
			if #[cfg(feature="checked-overflow")] {
				multiplicand.checked_mul(multiplier)
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(multiplicand.wrapping_mul(multiplier)))
			}
		}
	}
}

impl TryDiv for Number {
	type Error = MathError;
	type Output = Self;

	/// Attempts to divide `self` by `divisor`, returning the quotient (rounding towards zero).
	///
	/// # Errors
	/// If `divisor` is zero, this will return a [`MathError::DivisionByZero`].
	///
	/// Without the `checked-overflow` feature, this will [truncate](Number:new_truncate) the result. With the feature, a
	/// [`MathError::Overflow`] will be returned if the result is unable to fit within a [`Number`]. (Note that this
	/// will only happen if [`Number::MIN`] is divided by negative one.)
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Number, number::MathError, ops::TryDiv};
	/// let dividend = Number::new(123).unwrap();
	/// let divisor = Number::new(-12).unwrap();
	/// assert_eq!(dividend.try_div(divisor).unwrap().get(), -10);
	///
	/// // division by zero is an error.
	/// assert_eq!(dividend.try_div(Number::ZERO), Err(MathError::DivisionByZero));
	/// ```
	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_div(self, divisor: Self) -> Result<Self::Output, Self::Error> {
		let dividend = self.get();
		let divisor = divisor.get();

		if divisor == 0 {
			return Err(MathError::DivisionByZero);
		}

		cfg_if! {
			if #[cfg(feature="checked-overflow")] {
				dividend.checked_div(divisor)
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(dividend.wrapping_div(divisor)))
			}
		}
	}
}

impl TryRem for Number {
	type Error = MathError;
	type Output = Self;

	/// Attempts to modulo `self` by `divisor`, returning the remainder.
	///
	/// # Errors
	/// If `divisor` is zero, this will return a [`MathError::DivisionByZero`].
	/// If either `self` or `divisor` is negative, this will return a [`MathError::NegativeModulo`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Number, number::MathError, ops::TryRem};
	/// let dividend = Number::new(123).unwrap();
	/// let divisor = Number::new(12).unwrap();
	/// assert_eq!(dividend.try_rem(divisor).unwrap().get(), 3);
	///
	/// // modulo by zero is an error.
	/// assert_eq!(dividend.try_rem(Number::ZERO), Err(MathError::DivisionByZero));
	///
	/// // modulo with either value being negative is an error.
	/// let negative = Number::new(-15).unwrap();
	/// assert_eq!(dividend.try_rem(negative), Err(MathError::NegativeModulo));
	/// assert_eq!(negative.try_rem(divisor), Err(MathError::NegativeModulo));
	/// ```
	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_rem(self, divisor: Self) -> Result<Self::Output, Self::Error> {
		let dividend = self.get();
		let divisor = divisor.get();

		if divisor == 0 {
			return Err(MathError::DivisionByZero);
		} else if dividend < 0 || divisor < 0 {
			return Err(MathError::NegativeModulo);
		}

		// note that we don't check for overflow as negative modulo is not allowed.
		Ok(Self::new(dividend.wrapping_rem(divisor)).unwrap())
	}
}

impl TryPow for Number {
	type Error = MathError;
	type Output = Self;

	/// Attempts to raise `self` to the power of `exponent`, returning the exponentiation.
	///
	/// # Errors
	/// If `self` is zero and `exponent` is negative, this will return a [`MathError::DivisionByZero`].
	///
	/// Without the `checked-overflow` feature, this will [truncate](Number:new_truncate) the result. With the feature, a
	/// [`MathError::Overflow`] will be returned if the result is unable to fit within a [`Number`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Number, number::MathError, ops::TryPow};
	/// let base = Number::new(5).unwrap();
	/// let exponent = Number::new(12).unwrap();
	/// assert_eq!(base.try_pow(exponent).unwrap().get(), 244_140_625);
	///
	/// // raising zero to a negative power is an error
	/// assert_eq!(Number::ZERO.try_pow(Number::NEG_ONE), Err(MathError::DivisionByZero));
	/// ```
	fn try_pow(self, exponent: Self) -> Result<Self::Output, Self::Error> {
		let base = self.get();
		let exponent = exponent.get();

		if base == 0 && exponent < 0 {
			return Err(MathError::DivisionByZero);
		} else if base == 1 {
			return Ok(Self::ONE);
		}

		cfg_if! {
			if #[cfg(feature="checked-overflow")] {
				// if we're able to use the builtlin `checked_pow`, use that.
				if let Ok(exponent) = u32::try_from(exponent) {
					base.checked_pow(exponent)
						.and_then(Self::new)
						.ok_or(MathError::Overflow) 
				} else if exponent < 0 {
					// if the exponent is negative, then the result is 0 for all non-`-1` numbers (we handled `1` above).
					if base != -1 {
						Ok(Self::ZERO)
					} else if exponent & 1 == 1 {
						Ok(Self::NEG_ONE)
					} else {
						Ok(Self::ONE)
					}
				} else {
					debug_assert!(exponent > (u32::MAX as i64));
					Err(MathError::Overflow)
				}
			} else {
				// For all 32-bit numbers where the result of `x^y` is a 32-bit number, converting them to a `f64` and
				// exponentiating them via `f64::powf` will always be valid. As such, it's faster than doing it with `i64`s.
				Ok(Self::new_truncate((base as f64).powf(exponent as f64) as i64))
			}
		}
	}
}
