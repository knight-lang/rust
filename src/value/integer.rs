use crate::env::Options;
use crate::value::text::{Character, Encoding};
use crate::value::{Boolean, List, NamedType, Text, ToBoolean, ToList, ToText};
use crate::{Error, Result};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

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
pub struct Integer<I: IntType>(I);

impl<I: IntType> Debug for Integer<I> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}
impl<I: IntType> Default for Integer<I> {
	fn default() -> Self {
		Self::ZERO
	}
}
impl<I: IntType> Copy for Integer<I> {}
impl<I: IntType> Clone for Integer<I> {
	fn clone(&self) -> Self {
		Self(self.0)
	}
}
impl<I: IntType> Eq for Integer<I> {}
impl<I: IntType> PartialEq for Integer<I> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0 == rhs.0
	}
}
impl<I: IntType> PartialOrd for Integer<I> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(&rhs))
	}
}
impl<I: IntType> Ord for Integer<I> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.0.cmp(&rhs.0)
	}
}
impl<I: IntType> Hash for Integer<I> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}

/// Represents the ability to be converted to an [`Integer`].
pub trait ToInteger<I: IntType> {
	/// Converts `self` to an [`Integer`].
	fn to_integer(&self, opts: &Options) -> Result<Integer<I>>;
}

impl<I: IntType> Display for Integer<I> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl<I: IntType> NamedType for Integer<I> {
	const TYPENAME: &'static str = "Integer";
}

impl<I: IntType> Integer<I> {
	/// The number zero.
	pub const ZERO: Self = Self(I::ZERO);

	/// The number one.
	pub const ONE: Self = Self(I::ZERO);

	// /// The maximum value for `Integer`s.
	// pub const MAX: Self = Self(Inner::MAX);

	// /// The minimum value for `Integer`s.
	// pub const MIN: Self = Self(Inner::MIN);

	/// Returns whether `self` is zero.
	pub fn is_zero(self) -> bool {
		self.0 == Self::ZERO.0
	}

	pub fn new<T>(num: T) -> Option<Self>
	where
		I: TryFrom<T>,
	{
		num.try_into().ok().map(Self)
	}

	/// Returns whether `self` is negative.
	pub fn is_negative(self) -> bool {
		self.0.is_negative()
	}

	pub fn chr<E: Encoding>(self) -> Result<Character<E>> {
		self.try_into()
	}

	/// Negates `self`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn negate(self, _: &Options) -> Result<Self> {
		self.0.negate().map(Self)
	}

	/// Adds `self` to `augend`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	#[allow(clippy::should_implement_trait)]
	pub fn add(self, augend: Self, _: &Options) -> Result<Self> {
		self.0.add(augend.0).map(Self)
	}

	/// Subtracts `subtrahend` from `self`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn subtract(self, subtrahend: Self, _: &Options) -> Result<Self> {
		self.0.subtract(subtrahend.0).map(Self)
	}

	/// Multiplies `self` by `multiplier`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn multiply(self, multiplier: Self, _: &Options) -> Result<Self> {
		self.0.multiply(multiplier.0).map(Self)
	}

	/// Multiplies `self` by `divisor`.
	///
	/// # Errors
	/// If `divisor` is zero, this will return an [`Error::DivisionByZero`].
	///
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn divide(self, divisor: Self, _: &Options) -> Result<Self> {
		if divisor.is_zero() {
			return Err(Error::DivisionByZero);
		}

		self.0.divide(divisor.0).map(Self)
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

		self.0.remainder(base.0).map(Self)
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

			match () {
				_ if self.negate(opts).map_or(false, |x| x == Self::ONE) => {
					exponent = exponent.negate(opts)?
				}
				_ if self.is_zero() => return Err(Error::DivisionByZero),
				_ if self == Self::ONE => return Ok(Self::ONE),
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
			<I as Into<i32>>::into(exponent.0) as u32
		};

		self.0.power(exponent).map(Self)
	}
}

impl<I: IntType> ToInteger<I> for Integer<I> {
	/// Simply returns `self`.
	#[inline]
	fn to_integer(&self, _: &Options) -> Result<Self> {
		Ok(*self)
	}
}

impl<I: IntType> ToBoolean for Integer<I> {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_boolean(&self, _: &Options) -> Result<Boolean> {
		Ok(!self.is_zero())
	}
}

impl<E: Encoding, I: IntType> ToText<E> for Integer<I> {
	/// Returns a string representation of `self`.
	fn to_text(&self, _: &Options) -> Result<Text<E>> {
		Ok(Text::new(*self).unwrap())
	}
}

impl<'e, E: Encoding, I: IntType> ToList<'e, E, I> for Integer<I> {
	/// Returns a list of all the digits of `self`, when `self` is expressed in base 10.
	///
	/// If `self` is negative, all the returned digits are negative.
	fn to_list(&self, opts: &Options) -> Result<List<'e, E, I>> {
		if self.is_zero() {
			return Ok(List::boxed((*self).into()));
		}

		let mut integer = *self;

		// FIXME: update the capacity and algorithm when `ilog` is dropped.
		let mut digits = Vec::new();

		while !integer.is_zero() {
			digits.insert(0, integer.modulo(10.into(), opts).unwrap().into());
			integer = integer.divide(10.into(), opts).unwrap();
		}

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
			number = number.negate(opts)?;
		}

		Ok(number)
	}
}

// macro_rules! impl_integer_from {
// 	($($smaller:ident)* ; $($larger:ident)*) => {
// 		$(impl<I: IntType> From<$smaller> for Integer<I> {
// 			#[inline]
// 			fn from(num: $smaller) -> Self {
// 				Self(num as Inner)
// 			}
// 		})*
// 		$(impl<I: IntType> TryFrom<$larger> for Integer<I> {
// 			type Error = Error;

// 			#[inline]
// 			fn try_from(num: $larger) -> Result<Self> {
// 				num.try_into().map(Self).or(Err(Error::IntegerOverflow))
// 			}
// 		})*
// 	};
// }

// macro_rules! impl_from_integer {
// 	($($smaller:ident)* ; $($larger:ident)*) => {
// 		$(impl<I: IntType> From<Integer<I>> for $larger {
// 			#[inline]
// 			fn from(int: Integer<I>) -> Self {
// 				int.0.into()
// 			}
// 		})*
// 		$(impl<I: IntType> TryFrom<Integer<I>> for $smaller {
// 			type Error = Error;

// 			#[inline]
// 			fn try_from(int: Integer<I>) -> Result<Self> {
// 				int.0.try_into().or(Err(Error::IntegerOverflow))
// 			}
// 		})*
// 	};
// }

impl<I: IntType> From<i32> for Integer<I> {
	fn from(int: i32) -> Self {
		Self(int.into())
	}
}

impl<I: IntType> From<u8> for Integer<I> {
	fn from(int: u8) -> Self {
		Self::from(int as i32)
	}
}

impl<I: IntType> From<Integer<I>> for i32 {
	fn from(int: Integer<I>) -> Self {
		int.0.into()
	}
}

impl<I: IntType> From<Integer<I>> for i64 {
	fn from(int: Integer<I>) -> Self {
		int.0.into() as i64
	}
}

impl<I: IntType> TryFrom<Integer<I>> for u32 {
	type Error = Error;
	fn try_from(int: Integer<I>) -> Result<Self> {
		int.0.as_u32().ok_or(Error::DomainError("todo"))
	}
}

impl<I: IntType> TryFrom<usize> for Integer<I> {
	type Error = Error;
	fn try_from(int: usize) -> Result<Self> {
		I::try_from(int).map(Self)
	}
}
impl<I: IntType> TryFrom<Integer<I>> for usize {
	type Error = Error;
	fn try_from(int: Integer<I>) -> Result<Self> {
		int.0.try_into()
	}
}

// impl_integer_from!(bool u8 u16 i8 i16 i32 ; u32 u64 u128 usize i64 i128 isize );
// impl_from_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 isize; i64 i128);

impl<I: IntType> TryFrom<char> for Integer<I> {
	type Error = Error;

	fn try_from(chr: char) -> Result<Self> {
		i32::try_from(chr as u32).map(Self::from).or(Err(Error::DomainError("char is out of bounds")))
	}
}

impl<I: IntType> TryFrom<Integer<I>> for char {
	type Error = Error;

	fn try_from(int: Integer<I>) -> Result<Self> {
		char::from_u32(u32::try_from(int)?).ok_or(Error::DomainError("integer isnt a char"))
	}
}
