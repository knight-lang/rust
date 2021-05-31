use crate::{Value, Boolean, Text};
use crate::value::{SHIFT, Tag, ValueKind};
use crate::ops::{Runnable, ToText, TryAdd, TrySub, TryMul, TryDiv, TryRem, TryPow, TryNeg, Infallible};
use std::fmt::{self, Display, Formatter};
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

	/// The maximum value a [`Number`] can contain.
	///
	/// Note that when the `strict-numbers` feature is enabled, this is simply [`i32::MAX`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// #assert_eq!(Number::new(Number::MAX.get() - 1).unwrap(), Number::MAX.get() - 1);
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
	/// #assert_eq!(Number::new(Number::MIN.get() + 1).unwrap(), Number::MIN.get() + 1);
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
	/// assert_eq!(Number::new_truncate(Number::MIN.get() - 1).get(), -1);
	/// assert_eq!(Number::new_truncate(Number::MAX.get() + 1).get(), 0);
	/// ```
	#[inline]
	pub const fn new_truncate(num: i64) -> Self {
		#[cfg(feature = "strict-numbers")]
		{ Self(num & 0xffff_ffff) }

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
	/// assert_eq!(false, Number::ZERO.into());
	/// assert_eq!(true, Number::ONE.into());
	/// assert_eq!(true, Number::new(123).into());
	/// assert_eq!(true, Number::new(-1).into());
	/// ```
	#[inline]
	fn from(number: Number) -> Self {
		number.get() != 0
	}
}

impl ToText for Number {
	type Error = Infallible;
	type Output = Text;

	#[inline]
	fn to_text(&self) -> Result<Self::Output, Self::Error> {
		// TODO: use some form of caching here.
		Ok(Text::new(self.to_string().into()).unwrap())
	}
}

impl From<Number> for i64 {
	#[inline]
	fn from(number: Number) -> Self {
		number.get()
	}
}


/// An error indicating that a value was outside the allowable range for a number.
#[derive(Debug)]
pub struct OutOfBounds {
	_priv: ()
}

impl Display for OutOfBounds {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "the supplied value was not within bounds")
	}
}

impl std::error::Error for OutOfBounds {}

macro_rules! impl_from_smaller {
	($($infallible:ty),*; $($fallible:ty),*) => {
		$(
			impl From<$infallible> for Number {
				fn from(num: $infallible) -> Self {
					// SAFETY: `num` is always within range.
					unsafe {
						Self::new_unchecked(num as i64)
					}
				}
			}
		)*

		$(
			impl TryFrom<$fallible> for Number {
				type Error = OutOfBounds;

				fn try_from(num: $fallible) -> Result<Self, OutOfBounds> {
					i64::try_from(num)
						.ok()
						.and_then(Number::new)
						.ok_or(OutOfBounds { _priv: () })
				}
			}
		)*
	};
}

impl_from_smaller!(i8, u8, i16, u16, i32, u32; i64, u64, i128, u128);


#[derive(Debug)]
pub enum MathError {
	Overflow
}

impl TryAdd for Number {
	type Error = MathError;
	type Output = Self;

	fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
		let lhs = self.get();
		let rhs = rhs.get();

		cfg_if! {
			if #[cfg(feature="strict-numbers")] {
				lhs.checked_add(rhs)
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(lhs.wrapping_add(rhs)))
			}
		}
	}
}

// TODO: do we want to implement `Add` traits or somethin?

impl TrySub for Number {
	type Error = MathError;
	type Output = Self;

	fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
		let lhs = self.get();
		let rhs = rhs.get();

		cfg_if! {
			if #[cfg(feature="strict-numbers")] {
				lhs.checked_sub(rhs)
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(lhs.wrapping_sub(rhs)))
			}
		}
	}
}

impl TryMul for Number {
	type Error = MathError;
	type Output = Self;

	fn try_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
		let lhs = self.get();
		let rhs = rhs.get();

		cfg_if! {
			if #[cfg(feature="strict-numbers")] {
				lhs.checked_mul(rhs)
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(lhs.wrapping_mul(rhs)))
			}
		}
	}
}

impl TryDiv for Number {
	type Error = MathError;
	type Output = Self;

	fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
		let lhs = self.get();
		let rhs = rhs.get();

		cfg_if! {
			if #[cfg(feature="strict-numbers")] {
				lhs.checked_div(rhs)
					.and_then(Self::new)
					.ok_or(MathError::Overflow) 
			} else {
				Ok(Self::new_truncate(lhs.wrapping_div(rhs)))
			}
		}
	}
}

impl TryRem for Number {
	type Error = MathError;
	type Output = Self;

	fn try_rem(self, rhs: Self) -> Result<Self::Output, Self::Error> {
		let _ = rhs;
		todo!()
	}
}

impl TryNeg for Number {
	type Error = MathError;
	type Output = Self;
	fn try_neg(self) -> Result<Self::Output, Self::Error> {
		todo!()
	}
}

impl TryPow for Number {
	type Error = MathError;
	type Output = Self;
	fn try_pow(self, rhs: Self) -> Result<Self::Output, Self::Error> {
		let _ = rhs;
		todo!()
	}
}
