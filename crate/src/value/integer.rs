use crate::value::{Boolean, KnightType, List, Text, ToBoolean, ToList, ToText};
use crate::{Error, Result};
use std::fmt::{self, Display, Formatter};

/// The integer type within Knight.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer(Inner);

cfg_if! {
	if #[cfg(feature = "strict-integers")] {
		type Inner = i32;
	} else {
		type Inner = i64;
	}
}

/// Represents the ability to be converted to an [`Integer`].
pub trait ToInteger {
	/// Converts `self` to an [`Integer`].
	fn to_integer(&self) -> Result<Integer>;
}

impl Display for Integer {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl KnightType<'_> for Integer {
	const TYPENAME: &'static str = "Integer";
}

impl Integer {
	/// The number zero.
	pub const ZERO: Self = Self(0);

	/// The number one.
	pub const ONE: Self = Self(0);

	/// The maximum value for `Integer`s.
	pub const MAX: Self = Self(Inner::MAX);

	/// The minimum value for `Integer`s.
	pub const MIN: Self = Self(Inner::MIN);

	/// Returns whether `self` is zero.
	pub const fn is_zero(self) -> bool {
		self.0 == 0
	}

	pub const fn inner(self) -> Inner {
		self.0
	}

	/// Returns whether `self` is negative.
	pub const fn is_negative(self) -> bool {
		self.0.is_negative()
	}

	/// Negates `self`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn negate(self) -> Result<Self> {
		if cfg!(feature = "checked-overflow") {
			self.0.checked_neg().map(Self).ok_or(Error::IntegerOverflow)
		} else {
			Ok(Self(self.0.wrapping_neg()))
		}
	}

	fn binary_op<T>(
		self,
		rhs: T,
		checked: impl FnOnce(Inner, T) -> Option<Inner>,
		wrapping: impl FnOnce(Inner, T) -> Inner,
	) -> Result<Self> {
		if cfg!(feature = "checked-overflow") {
			(checked)(self.0, rhs).map(Self).ok_or(Error::IntegerOverflow)
		} else {
			Ok(Self((wrapping)(self.0, rhs)))
		}
	}

	/// Adds `self` to `augend`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	#[allow(clippy::should_implement_trait)]
	pub fn add(self, augend: Self) -> Result<Self> {
		self.binary_op(augend.0, Inner::checked_add, Inner::wrapping_add)
	}

	/// Subtracts `subtrahend` from `self`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn subtract(self, subtrahend: Self) -> Result<Self> {
		self.binary_op(subtrahend.0, Inner::checked_sub, Inner::wrapping_sub)
	}

	/// Multiplies `self` by `multiplier`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn multiply(self, multiplier: Self) -> Result<Self> {
		self.binary_op(multiplier.0, Inner::checked_mul, Inner::wrapping_mul)
	}

	/// Multiplies `self` by `divisor`.
	///
	/// # Errors
	/// If `divisor` is zero, this will return an [`Error::DivisionByZero`].
	///
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn divide(self, divisor: Self) -> Result<Self> {
		if divisor.is_zero() {
			return Err(Error::DivisionByZero);
		}

		self.binary_op(divisor.0, Inner::checked_div, Inner::wrapping_div)
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
	pub fn modulo(self, base: Self) -> Result<Self> {
		if base.is_zero() {
			return Err(Error::DivisionByZero);
		}

		if cfg!(feature = "strict-integers") && base.is_negative() {
			return Err(Error::DomainError("modulo by a negative base"));
		}

		self.binary_op(base.0, Inner::checked_rem, Inner::wrapping_rem)
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
	pub fn power(self, mut exponent: Self) -> Result<Self> {
		if exponent.is_negative() {
			if cfg!(feature = "strict-integers") {
				return Err(Error::DomainError("negative exponent"));
			}

			match self.0 {
				-1 => exponent = exponent.negate()?,
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
		let exponent = if cfg!(feature = "checked-overflow") {
			u32::try_from(exponent).or(Err(Error::DomainError("exponent too large")))?
		} else {
			exponent.0 as u32
		};

		self.binary_op(exponent, Inner::checked_pow, Inner::wrapping_pow)
	}
}

impl ToInteger for Integer {
	/// Simply returns `self`.
	#[inline]
	fn to_integer(&self) -> Result<Self> {
		Ok(*self)
	}
}

impl ToBoolean for Integer {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_boolean(&self) -> Result<Boolean> {
		Ok(!self.is_zero())
	}
}

impl ToText for Integer {
	/// Returns a string representation of `self`.
	fn to_text(&self) -> Result<Text> {
		Ok(Text::new(*self).unwrap())
	}
}

impl<'e> ToList<'e> for Integer {
	/// Returns a list of all the digits of `self`, when `self` is expressed in base 10.
	fn to_list(&self) -> Result<List<'e>> {
		if self.is_zero() {
			return Ok(List::boxed((*self).into()));
		}

		let mut integer = self.0;

		if integer.is_negative() {
			panic!("todo?");
			// integer = integer.negate()?; <-- wont work because it's actually valid.
		}

		// FIXME: update the capacity _and_ algorithm when `ilog` is dropped.
		let mut digits = Vec::new();

		while integer != 0 {
			digits.push(Self(integer % 10).into());
			integer /= 10;
		}

		digits.reverse();

		Ok(digits.try_into().unwrap())
	}
}

impl std::str::FromStr for Integer {
	type Err = Error;

	fn from_str(inp: &str) -> Result<Self> {
		let mut bytes = inp.trim_start().bytes();

		let (is_negative, mut number) = match bytes.next() {
			Some(b'+') => (false, Integer::ZERO),
			Some(b'-') => (true, Integer::ZERO),
			Some(num @ b'0'..=b'9') => (false, Integer::from(num - b'0')),
			_ => return Ok(Integer::ZERO),
		};

		while let Some(digit @ b'0'..=b'9') = bytes.next() {
			number = number.multiply(10.into())?.add((digit - b'0').into())?;
		}

		if is_negative {
			number.negate()
		} else {
			Ok(number)
		}
	}
}

macro_rules! impl_integer_from {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl From<$smaller> for Integer {
			fn from(num: $smaller) -> Self {
				Self(num as Inner)
			}
		})*
		$(impl TryFrom<$larger> for Integer {
			type Error = Error;

			fn try_from(num: $larger) -> Result<Self> {
				num.try_into().map(Self).or(Err(Error::IntegerOverflow))
			}
		})*
	};
}

macro_rules! impl_from_integer {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl From<Integer> for $larger {
			fn from(int: Integer) -> Self {
				int.0.into()
			}
		})*
		$(impl TryFrom<Integer> for $smaller {
			type Error = Error;

			fn try_from(int: Integer) -> Result<Self> {
				int.0.try_into().or(Err(Error::IntegerOverflow))
			}
		})*
	};
}

impl_integer_from!(bool u8 u16 i8 i16 i32 ; u32 u64 u128 usize i64 i128 isize );
impl_from_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 isize; i64 i128);

impl TryFrom<char> for Integer {
	type Error = Error;

	fn try_from(chr: char) -> Result<Self> {
		(chr as u32).try_into()
	}
}

impl TryFrom<Integer> for char {
	type Error = Error;

	fn try_from(int: Integer) -> Result<Self> {
		char::from_u32(u32::try_from(int)?).ok_or(Error::DomainError("integer isnt a char"))
	}
}
