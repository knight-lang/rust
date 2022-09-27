use crate::env::Options;
use crate::value::text::{Character, Encoding};
use crate::value::{Boolean, List, NamedType, Text, ToBoolean, ToList, ToText};
use crate::{Error, Result};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

mod int_type;
pub use int_type::*;

/// The integer type within Knight.
///
/// # Bit Size
/// According to the knight spec, integers must be within the range `-2147483648..2147483647i32`
/// (inclusive on both sides), i.e. a `i32`'s bounds. However, implementations are free to go above
/// that number. So, this implementation defaults to an [`i64`] as its internal integer size, and
/// will switch to [`i32`] if the `strict-integers` feature is enabled.
///
/// # Conversions
/// Since the internal representation can either be a 32 or 64 bit integer, all conversions are
/// implemented as though the internal type is a 32 bit integer.
///
/// # Overflow operations
/// Within Knight, any integer operations which under/overflow the bounds of a 32 bit integer are
/// undefined. Within this implementation, all operations normally use wrapping logic. However, if
/// the `checked-overflow` feature is enabled, an [`Error::IntegerOverflow`] is returned whenever
/// an operation would overflow.
pub struct Integer<I>(Inner, PhantomData<I>);

impl<I> Debug for Integer<I> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}
impl<I> Default for Integer<I> {
	fn default() -> Self {
		Self::ZERO
	}
}
impl<I> Copy for Integer<I> {}
impl<I> Clone for Integer<I> {
	fn clone(&self) -> Self {
		Self(self.0, PhantomData)
	}
}
impl<I> Eq for Integer<I> {}
impl<I> PartialEq for Integer<I> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0 == rhs.0
	}
}
impl<I> PartialOrd for Integer<I> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(&rhs))
	}
}
impl<I> Ord for Integer<I> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.0.cmp(&rhs.0)
	}
}
impl<I> Hash for Integer<I> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}

#[cfg(feature = "strict-integers")]
type Inner = i32;

#[cfg(not(feature = "strict-integers"))]
type Inner = i64;

/// Represents the ability to be converted to an [`Integer`].
pub trait ToInteger<I> {
	/// Converts `self` to an [`Integer`].
	fn to_integer(&self, opts: &Options) -> Result<Integer<I>>;
}

impl<I> Display for Integer<I> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl<I> NamedType for Integer<I> {
	const TYPENAME: &'static str = "Integer";
}

impl<I> Integer<I> {
	/// The number zero.
	pub const ZERO: Self = Self(0, PhantomData);

	/// The number one.
	pub const ONE: Self = Self(0, PhantomData);

	// /// The maximum value for `Integer`s.
	// pub const MAX: Self = Self(Inner::MAX, PhantomData);

	// /// The minimum value for `Integer`s.
	// pub const MIN: Self = Self(Inner::MIN, PhantomData);

	/// Returns whether `self` is zero.
	pub const fn is_zero(self) -> bool {
		self.0 == 0
	}

	pub const fn new(num: i64) -> Option<Self> {
		// if opts.compliance.i32_integer && (num < i32::MIN as i64 || num > i32::MAX as i64) {
		// 	None
		// } else {
		Some(Self(num, PhantomData))
		// }
	}

	/// Returns whether `self` is negative.
	pub const fn is_negative(self) -> bool {
		self.0.is_negative()
	}

	pub fn chr<E: Encoding>(self) -> Result<Character<E>> {
		self.try_into()
	}
}

impl<I: IntType> Integer<I> {
	/// Negates `self`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn negate(self) -> Result<Self> {
		I::negate(self)
	}
}

impl<I> Integer<I> {
	fn binary_op<T>(
		self,
		rhs: T,
		opts: &Options,
		checked: impl FnOnce(Inner, T) -> Option<Inner>,
		wrapping: impl FnOnce(Inner, T) -> Inner,
	) -> Result<Self> {
		if opts.compliance.checked_overflow {
			(checked)(self.0, rhs).map(|x| Self(x, PhantomData)).ok_or(Error::IntegerOverflow)
		} else {
			Ok(Self((wrapping)(self.0, rhs), PhantomData))
		}
	}

	/// Adds `self` to `augend`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	#[allow(clippy::should_implement_trait)]
	pub fn add(self, augend: Self, opts: &Options) -> Result<Self> {
		self.binary_op(augend.0, opts, Inner::checked_add, Inner::wrapping_add)
	}

	/// Subtracts `subtrahend` from `self`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn subtract(self, subtrahend: Self, opts: &Options) -> Result<Self> {
		self.binary_op(subtrahend.0, opts, Inner::checked_sub, Inner::wrapping_sub)
	}

	/// Multiplies `self` by `multiplier`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn multiply(self, multiplier: Self, opts: &Options) -> Result<Self> {
		self.binary_op(multiplier.0, opts, Inner::checked_mul, Inner::wrapping_mul)
	}

	/// Multiplies `self` by `divisor`.
	///
	/// # Errors
	/// If `divisor` is zero, this will return an [`Error::DivisionByZero`].
	///
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn divide(self, divisor: Self, opts: &Options) -> Result<Self> {
		if divisor.is_zero() {
			return Err(Error::DivisionByZero);
		}

		self.binary_op(divisor.0, opts, Inner::checked_div, Inner::wrapping_div)
	}

	/// Returns `self` modulo `base`.
	///
	/// # Errors
	/// If `base` is zero, this will return an [`Error::DivisionByZero`].
	///
	/// If `base` is negative and `strict-integers` is enabled, [`Error::DomainError`] is returned.
	///
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn modulo(self, base: Self, opts: &Options) -> Result<Self> {
		if base.is_zero() {
			return Err(Error::DivisionByZero);
		}

		if opts.compliance.check_modulo_argument && self.is_negative() || base.is_negative() {
			return Err(Error::DomainError("modulo by a negative base"));
		}

		self.binary_op(base.0, opts, Inner::checked_rem, Inner::wrapping_rem)
	}

	/// Raises `self` to the `exponent` power.
	///
	/// # Errors
	/// If the exponent is negative and `strict-integers` is enabled, an [`Error::DomainError`]
	/// is returned.
	///
	/// If `strict-integers` is not enabled, the exponent is negative, and `self` is zero, then
	/// an [`Error::DivisionByZero`] is returned.
	///
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn power(self, mut exponent: Self, opts: &Options) -> Result<Self> {
		if exponent.is_negative() {
			if opts.compliance.check_power_argument {
				return Err(Error::DomainError("negative exponent"));
			}

			match self.0 {
				-1 => exponent = exponent.negate(opts)?,
				0 => return Err(Error::DivisionByZero),
				1 => return Ok(Self::ONE),
				_ => return Ok(Self::ZERO),
			}
		}

		if exponent.is_zero() {
			return Ok(Self::ONE);
		}

		if self.is_zero() || self == Self::ONE {
			return Ok(self);
		}

		// FIXME: you could probably optimize this.
		let exponent = if opts.compliance.checked_overflow {
			u32::try_from(exponent).or(Err(Error::DomainError("exponent too large")))?
		} else {
			exponent.0 as u32
		};

		self.binary_op(exponent, opts, Inner::checked_pow, Inner::wrapping_pow)
	}
}

impl<I> ToInteger<I> for Integer<I> {
	/// Simply returns `self`.
	#[inline]
	fn to_integer(&self, _: &Options) -> Result<Self> {
		Ok(*self)
	}
}

impl<I> ToBoolean for Integer<I> {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_boolean(&self, _: &Options) -> Result<Boolean> {
		Ok(!self.is_zero())
	}
}

impl<E: Encoding, I> ToText<E> for Integer<I> {
	/// Returns a string representation of `self`.
	fn to_text(&self, _: &Options) -> Result<Text<E>> {
		Ok(Text::new(*self).unwrap())
	}
}

impl<'e, E, I> ToList<'e, E, I> for Integer<I> {
	/// Returns a list of all the digits of `self`, when `self` is expressed in base 10.
	///
	/// If `self` is negative, all the returned digits are negative.
	fn to_list(&self, _: &Options) -> Result<List<'e, E, I>> {
		if self.is_zero() {
			return Ok(List::boxed((*self).into()));
		}

		let mut integer = self.0;

		// FIXME: update the capacity and algorithm when `ilog` is dropped.
		let mut digits = Vec::new();

		while integer != 0 {
			digits.push(Self(integer % 10, PhantomData).into());
			integer /= 10;
		}

		digits.reverse();

		Ok(digits.try_into().unwrap())
	}
}

impl<I: IntType> Integer<I> {
	pub fn parse(input: &str, opts: &Options) -> Result<Self> {
		let mut bytes = input.trim_start().bytes();

		let (is_negative, mut number) = match bytes.next() {
			Some(b'+') => (false, Integer::ZERO),
			Some(b'-') => (true, Integer::ZERO),
			Some(digit @ b'0'..=b'9') => (false, Integer::from(digit - b'0')),
			_ => return Ok(Integer::ZERO),
		};

		while let Some(digit @ b'0'..=b'9') = bytes.next() {
			number = number.multiply(10.into(), opts)?.add((digit - b'0').into(), opts)?;
		}

		if is_negative {
			number = number.negate()?;
		}

		Ok(number)
	}
}

macro_rules! impl_integer_from {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl<I> From<$smaller> for Integer<I> {
			#[inline]
			fn from(num: $smaller) -> Self {
				Self(num as Inner, PhantomData)
			}
		})*
		$(impl<I> TryFrom<$larger> for Integer<I> {
			type Error = Error;

			#[inline]
			fn try_from(num: $larger) -> Result<Self> {
				num.try_into().map(|x| Self(x, PhantomData)).or(Err(Error::IntegerOverflow))
			}
		})*
	};
}

macro_rules! impl_from_integer {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl<I> From<Integer<I>> for $larger {
			#[inline]
			fn from(int: Integer<I>) -> Self {
				int.0.into()
			}
		})*
		$(impl<I> TryFrom<Integer<I>> for $smaller {
			type Error = Error;

			#[inline]
			fn try_from(int: Integer<I>) -> Result<Self> {
				int.0.try_into().or(Err(Error::IntegerOverflow))
			}
		})*
	};
}

impl_integer_from!(bool u8 u16 i8 i16 i32 ; u32 u64 u128 usize i64 i128 isize );
impl_from_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 isize; i64 i128);

impl<I> TryFrom<char> for Integer<I> {
	type Error = Error;

	fn try_from(chr: char) -> Result<Self> {
		(chr as u32).try_into()
	}
}

impl<I> TryFrom<Integer<I>> for char {
	type Error = Error;

	fn try_from(int: Integer<I>) -> Result<Self> {
		char::from_u32(u32::try_from(int)?).ok_or(Error::DomainError("integer isnt a char"))
	}
}
