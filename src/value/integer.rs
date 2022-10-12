use crate::parse::{self, Parsable, Parser};
use crate::value::text::{Character, Encoding};
use crate::value::{Boolean, List, NamedType, Text, ToBoolean, ToList, ToText};
use crate::{Environment, Error, Result};
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

mod inttype;
pub use inttype::{Checked, IntType, Wrapping};

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
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer<I>(I);

/// Represents the ability to be converted to an [`Integer`].
pub trait ToInteger<I, E> {
	/// Converts `self` to an [`Integer`].
	fn to_integer(&self, env: &mut Environment<I, E>) -> Result<Integer<I>>;
}

impl<I: Debug> Debug for Integer<I> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl<I: Display> Display for Integer<I> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl<I> NamedType for Integer<I> {
	const TYPENAME: &'static str = "Integer";
}

impl<I: IntType> Integer<I> {
	pub const fn new(int: I) -> Self {
		Self(int)
	}

	pub const ZERO: Self = Self(I::ZERO);
	pub const ONE: Self = Self(I::ONE);

	/// Returns whether `self` is zero.
	pub fn is_zero(self) -> bool {
		self.0 == I::ZERO
	}

	/// Returns whether `self` is negative.
	pub fn is_negative(self) -> bool {
		self.0 < I::ZERO
	}

	pub fn negate(self) -> Result<Self> {
		self.0.negate().map(Self)
	}

	pub fn add(self, rhs: Self) -> Result<Self> {
		self.0.add(rhs.0).map(Self)
	}
	pub fn subtract(self, rhs: Self) -> Result<Self> {
		self.0.subtract(rhs.0).map(Self)
	}
	pub fn multiply(self, rhs: Self) -> Result<Self> {
		self.0.multiply(rhs.0).map(Self)
	}
	pub fn divide(self, rhs: Self) -> Result<Self> {
		if rhs.is_zero() {
			return Err(Error::DivisionByZero);
		}

		self.0.divide(rhs.0).map(Self)
	}

	pub fn remainder(self, base: Self, flags: &crate::env::Flags) -> Result<Self> {
		if base.is_zero() {
			return Err(Error::DivisionByZero);
		}

		#[cfg(feature = "compliance")]
		if flags.compliance.check_integer_function_bounds {
			if self.is_negative() {
				return Err(Error::DomainError("remainder with a negative number"));
			}

			if base.is_negative() {
				return Err(Error::DomainError("remainder by a negative base"));
			}
		}

		let _ = flags;
		self.0.remainder(base.0).map(Self)
	}
	pub fn power(self, mut exponent: Self, flags: &crate::env::Flags) -> Result<Self> {
		if exponent.is_negative() {
			match self.0.into() {
				#[cfg(feature = "compliance")]
				_ if flags.compliance.check_integer_function_bounds => {
					return Err(Error::DomainError("negative exponent"))
				}
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
		#[allow(unused_mut)]
		let mut exp = exponent.0.into() as u32;

		#[cfg(feature = "compliance")]
		if flags.compliance.check_integer_function_bounds {
			exp = u32::try_from(exponent).or(Err(Error::DomainError("exponent too large")))?
		}

		let _ = flags;
		self.0.power(exp).map(Self)
	}
	pub fn log10(self) -> usize {
		self.0.log10()
	}

	/// Attempts to interpret `self` as a Unicode codepoint.
	pub fn chr<E: Encoding>(self) -> Result<Character<E>> {
		u32::try_from(self.0.into())
			.ok()
			.and_then(char::from_u32)
			.and_then(Character::new)
			.ok_or(Error::DomainError("number isn't a valid char"))
	}

	pub fn head(self) -> Self {
		todo!()

		// let mut n = self.0;
		// while 10 <= n.abs() {
		// 	n /= 10;
		// }
		// Self(n)
	}

	pub fn tail(self) -> Self {
		// Self(self.0 % 10)
		todo!()
	}

	pub fn random<R: rand::Rng + ?Sized>(rng: &mut R, flags: &crate::env::Flags) -> Self {
		#[allow(unused_mut)]
		let mut rand = rng.gen::<i64>().abs();

		#[cfg(feature = "compliance")]
		if flags.compliance.limit_rand_range {
			rand &= 0x7fff;
		}

		let _ = flags;

		// jank and needs to be fixed
		Self(rand.try_into().unwrap_or_else(|_| {
			let mut x = I::from(rand as i32);
			if x < I::ZERO {
				x = x.negate().unwrap_or_default();
			}
			x
		}))
	}
}

#[cfg(any())]
impl Integer {
	// /// The maximum value for `Integer`s.
	// pub const MAX: Self = Self(Inner::MAX);

	// /// The minimum value for `Integer`s.
	// pub const MIN: Self = Self(Inner::MIN);

	/// Negates `self`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	#[cfg_attr(not(feature = "checked-math-ops"), inline)]
	pub fn negate(self) -> Result<Self> {
		if cfg!(feature = "checked-math-ops") {
			return Ok(Self(self.0.wrapping_neg()));
		}

		self.0.checked_neg().map(Self).ok_or(Error::IntegerOverflow)
	}

	fn binary_op<T>(
		self,
		rhs: T,
		checked: impl FnOnce(Inner, T) -> Option<Inner>,
		wrapping: impl FnOnce(Inner, T) -> Inner,
	) -> Result<Self> {
		if cfg!(feature = "checked-math-ops") {
			return Ok(Self(wrapping(self.0, rhs)));
		}

		checked(self.0, rhs).map(Self).ok_or(Error::IntegerOverflow)
	}

	/// Adds `self` to `augend`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	#[allow(clippy::should_implement_trait)]
	#[cfg_attr(not(feature = "checked-math-ops"), inline)]
	pub fn add(self, augend: Self) -> Result<Self> {
		self.binary_op(augend.0, Inner::checked_add, Inner::wrapping_add)
	}

	/// Subtracts `subtrahend` from `self`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	#[cfg_attr(not(feature = "checked-math-ops"), inline)]
	pub fn subtract(self, subtrahend: Self) -> Result<Self> {
		self.binary_op(subtrahend.0, Inner::checked_sub, Inner::wrapping_sub)
	}

	/// Multiplies `self` by `multiplier`.
	///
	/// # Errors
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	#[cfg_attr(not(feature = "checked-math-ops"), inline)]
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
	#[cfg_attr(not(feature = "checked-math-ops"), inline)]
	pub fn divide(self, divisor: Self) -> Result<Self> {
		if divisor.is_zero() {
			return Err(Error::DivisionByZero);
		}

		self.binary_op(divisor.0, Inner::checked_div, Inner::wrapping_div)
	}

	/// Returns the remainder of `self` and `base`.
	///
	/// # Errors
	/// If `base` is zero, this will return an [`Error::DivisionByZero`].
	///
	/// If `base` is negative and `strict-integers` is enabled, [`Error::DomainError`] is returned.
	///
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	#[cfg_attr(not(feature = "checked-math-ops"), inline)]
	pub fn remainder(self, base: Self, flags: &crate::env::Flags) -> Result<Self> {
		if base.is_zero() {
			return Err(Error::DivisionByZero);
		}

		#[cfg(feature = "compliance")]
		if flags.compliance.check_integer_function_bounds {
			if self.is_negative() {
				return Err(Error::DomainError("remainder with a negative number"));
			}

			if base.is_negative() {
				return Err(Error::DomainError("remainder by a negative base"));
			}
		}

		let _ = flags;
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
	pub fn power(self, mut exponent: Self, flags: &crate::env::Flags) -> Result<Self> {
		if exponent.is_negative() {
			match self.0 {
				#[cfg(feature = "compliance")]
				_ if flags.compliance.check_integer_function_bounds => {
					return Err(Error::DomainError("negative exponent"))
				}
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
		#[allow(unused_mut)]
		let mut exp = exponent.0 as u32;

		#[cfg(feature = "compliance")]
		if flags.compliance.check_integer_function_bounds {
			exp = u32::try_from(exponent).or(Err(Error::DomainError("exponent too large")))?
		}

		let _ = flags;
		self.binary_op(exp, Inner::checked_pow, Inner::wrapping_pow)
	}
}

impl<I: IntType, E: Encoding> Parsable<I, E> for Integer<I> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> parse::Result<Option<Self>> {
		let Some(source) = parser.take_while(Character::is_numeric) else {
			return Ok(None);
		};

		source
			.parse::<Self>()
			.map(Some)
			.map_err(|_| parser.error(parse::ErrorKind::IntegerLiteralOverflow))
	}
}

impl<I: Clone, E> ToInteger<I, E> for Integer<I> {
	/// Simply returns `self`.
	#[inline]
	fn to_integer(&self, _: &mut Environment<I, E>) -> Result<Self> {
		Ok(self.clone())
	}
}

impl<I: IntType, E> ToBoolean<I, E> for Integer<I> {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_boolean(&self, _: &mut Environment<I, E>) -> Result<Boolean> {
		Ok(!self.is_zero())
	}
}

impl<I: Display, E: Encoding> ToText<I, E> for Integer<I> {
	/// Returns a string representation of `self`.
	#[inline]
	fn to_text(&self, env: &mut Environment<I, E>) -> Result<Text<E>> {
		Ok(Text::new(self, env.flags()).expect("`to_text for Integer failed?`"))
	}
}

impl<I: IntType, E> ToList<I, E> for Integer<I> {
	/// Returns a list of all the digits of `self`, when `self` is expressed in base 10.
	///
	/// If `self` is negative, all the returned digits are negative.
	fn to_list(&self, _: &mut Environment<I, E>) -> Result<List<I, E>> {
		if self.is_zero() {
			return Ok(List::boxed(self.clone().into()));
		}

		let mut integer: i64 = self.0.into();
		let mut digits = Vec::with_capacity(self.log10());

		while integer != 0 {
			digits.insert(0, Self(I::from((integer % 10) as i32)).into());
			integer /= 10;
		}

		// The maximum amount of digits for an Integer is vastly smaller than `i32::MAX`, so
		// there's no need to do a check.
		Ok(unsafe { List::new_unchecked(digits) })
	}
}

impl<I: IntType> FromStr for Integer<I> {
	type Err = Error;

	fn from_str(source: &str) -> Result<Self> {
		let source = source.trim_start();

		if source.is_empty() {
			return Ok(Self::default());
		}

		let mut start = source;
		if "+-".contains(source.as_bytes()[0] as char) {
			let mut c = start.chars();
			c.next();
			start = c.as_str();
		}

		if let Some(bad) = start.find(|c: char| !c.is_ascii_digit()) {
			start = &source[..bad + (start != source) as usize];
		} else if start != source {
			start = source;
		}

		Ok(Self(I::from_str(start).unwrap_or_default()))
	}
}

macro_rules! impl_integer_from {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl<I: IntType> From<$smaller> for Integer<I> {
			#[inline]
			fn from(num: $smaller) -> Self {
				Self(I::from(num as i32))
			}
		})*
		$(impl<I: IntType> TryFrom<$larger> for Integer<I> {
			type Error = Error;

			#[inline]
			fn try_from(num: $larger) -> Result<Self> {
				i64::try_from(num).ok().and_then(|x| I::try_from(x).ok()).map(Self).ok_or(Error::IntegerOverflow)
			}
		})*
	};
}

macro_rules! impl_from_integer {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl<I: IntType> From<Integer<I>> for $larger {
			#[inline]
			fn from(int: Integer<I>) -> Self {
				int.0.into() as _
			}
		})*
		$(impl<I: IntType> TryFrom<Integer<I>> for $smaller {
			type Error = Error;

			#[inline]
			fn try_from(int: Integer<I>) -> Result<Self> {
				int.0.try_into().ok().and_then(|x| x.try_into().ok()).ok_or(Error::IntegerOverflow)
			}
		})*
	};
}

impl_integer_from!(bool u8 u16 i8 i16 i32 ; u32 u64 u128 usize i64 i128 isize );
impl_from_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 isize; i64 i128);

impl<I: IntType> TryFrom<char> for Integer<I> {
	type Error = Error;

	#[inline]
	fn try_from(chr: char) -> Result<Self> {
		(chr as u32).try_into()
	}
}

impl<I: IntType> TryFrom<Integer<I>> for char {
	type Error = Error;

	fn try_from(int: Integer<I>) -> Result<Self> {
		char::from_u32(u32::try_from(int)?).ok_or(Error::DomainError("integer isn't a char"))
	}
}
