use crate::env::{Environment, Flags};
use crate::parse::{self, Parsable, Parser};
use crate::value::text::{Character, Encoding};
use crate::value::{Boolean, List, NamedType, Text, ToBoolean, ToList, ToText};
use crate::{Error, Result};
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

mod inttype;
pub use inttype::{Checked, IntType, Wrapping};

/// The integer type within Knight.
///
/// # Bit Size
/// According to the knight spec, integers must be within the range `-2147483648..=2147483647i32`,
/// ie an `i32`'s bounds. however, implementations are free to go beyond that range. As such, this
/// implementation provides the ability to use _either_ [`i32`]s or [`i64`]s as your integer type.
/// In fact, you can use any type, as long as it implements the [`IntType`] interface.
///
/// Additionally, since the Knight specs state that all operations on integers that would overflow/
/// underflow the bounds of an `i32` are undefined,two optoins are provided: [`Checked`] and
/// [`Wrapping`]. The [`Checked`] type will raise an error if its argument overflows, whereas the
/// [`Wrapping`] type will simply wraparound.
///
/// # Conversions
/// Since the internal representation is a minimum of `i32`, all conversions are implemented
/// assuming the base type is an `i32`.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer<I>(I);

/// Represents the ability to be converted to an [`Integer`].
pub trait ToInteger<I, E> {
	/// Converts `self` to an [`Integer`].
	fn to_integer(&self, env: &mut Environment<I, E>) -> Result<Integer<I>>;
}

impl<I: Debug> Debug for Integer<I> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl<I: Display> Display for Integer<I> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl<I> NamedType for Integer<I> {
	const TYPENAME: &'static str = "Integer";
}

impl<I> Integer<I> {
	/// Creates a new `Integer`.
	pub const fn new(int: I) -> Self {
		Self(int)
	}
}

impl<I: IntType> Integer<I> {
	/// The zero value.
	pub const ZERO: Self = Self(I::ZERO);

	/// The one value.
	pub const ONE: Self = Self(I::ONE);

	/// Returns whether `self` is zero.
	pub fn is_zero(self) -> bool {
		self.0 == I::ZERO
	}

	/// Returns whether `self` is negative.
	pub fn is_negative(self) -> bool {
		self.0 < I::ZERO
	}

	/// Negates `self`.
	///
	/// # Errors
	/// Returns any errors [`I::negate`](IntType::negate) returns.
	pub fn negate(self, flags: &Flags) -> Result<Self> {
		self.0.negate(flags).map(Self)
	}

	/// Adds `self` with `augend`.
	///
	/// # Errors
	/// Returns any errors [`I::add`](IntType::add) returns.
	pub fn add(self, augend: Self, flags: &Flags) -> Result<Self> {
		self.0.add(augend.0, flags).map(Self)
	}

	/// Subtracts `self` by `subtrahend`.
	///
	/// # Errors
	/// Returns any errors [`I::subtract`](IntType::subtract) returns.
	pub fn subtract(self, subtrahend: Self, flags: &Flags) -> Result<Self> {
		self.0.subtract(subtrahend.0, flags).map(Self)
	}

	/// Multiplies `self` by `multiplier`.
	///
	/// # Errors
	/// Returns any errors [`I::multiply`](IntType::multiply) returns.
	pub fn multiply(self, multiplier: Self, flags: &Flags) -> Result<Self> {
		self.0.multiply(multiplier.0, flags).map(Self)
	}

	/// Divides `self` by `multiplier`.
	///
	/// # Errors
	/// Returns [`Error::DivisionByZero`] if `divisor` is zero. Additionally, returns any errors
	/// [`I::divide`](IntType::divide) returns.
	pub fn divide(self, divisor: Self, flags: &Flags) -> Result<Self> {
		if divisor.is_zero() {
			return Err(Error::DivisionByZero);
		}

		self.0.divide(divisor.0, flags).map(Self)
	}

	/// Gets the remainder of `self` and `base`.
	///
	/// # Errors
	/// Returns [`Error::DivisionByZero`] if `divisor` is zero. If [`check_integer_function_bounds`](
	/// crate::env::flags::ComplianceFlags::check_integer_function_bounds) is enabled and either
	/// `self` or `rhs` is negative, an [`Error::DomainError`] is returned. Additionally, returns any
	/// errors [`I::remainder`](IntType::remainder) returns.
	pub fn remainder(self, base: Self, flags: &Flags) -> Result<Self> {
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

		self.0.remainder(base.0, flags).map(Self)
	}

	pub fn power(self, mut exponent: Self, flags: &Flags) -> Result<Self> {
		if exponent.is_negative() {
			match self.0.into() {
				#[cfg(feature = "compliance")]
				_ if flags.compliance.check_integer_function_bounds => {
					return Err(Error::DomainError("negative exponent"))
				}
				-1 => exponent = exponent.negate(flags)?,
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

		self.0.power(exp, flags).map(Self)
	}

	pub fn log10(self) -> usize {
		self.0.log10()
	}

	/// Attempts to interpret `self` as a utf8 codepoint.
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

	pub fn random<R: rand::Rng + ?Sized>(rng: &mut R, flags: &Flags) -> Self {
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
				x = x.negate(flags).unwrap_or_default();
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

	/// Returns the remainder of `self` and `base`.
	///
	/// # Errors
	/// If `base` is zero, this will return an [`Error::DivisionByZero`].
	///
	/// If `base` is negative and `strict-integers` is enabled, [`Error::DomainError`] is returned.
	///
	/// If the `checked-overflow` feature is enabled, this will return an [`Error::IntegerOverflow`]
	/// if the operation would overflow. If the feature isn't enabled, the wrapping variant is used.
	pub fn remainder(self, base: Self, flags: &Flags) -> Result<Self> {
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
	pub fn power(self, mut exponent: Self, flags: &Flags) -> Result<Self> {
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
	fn to_integer(&self, _: &mut Environment<I, E>) -> Result<Self> {
		Ok(self.clone())
	}
}

impl<I: IntType, E> ToBoolean<I, E> for Integer<I> {
	/// Returns whether `self` is nonzero.
	fn to_boolean(&self, _: &mut Environment<I, E>) -> Result<Boolean> {
		Ok(!self.is_zero())
	}
}

impl<I: Display, E: Encoding> ToText<I, E> for Integer<I> {
	/// Returns a string representation of `self`.
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
			fn from(num: $smaller) -> Self {
				Self(I::from(num as i32))
			}
		})*
		$(impl<I: IntType> TryFrom<$larger> for Integer<I> {
			type Error = Error;

			fn try_from(num: $larger) -> Result<Self> {
				i64::try_from(num).ok().and_then(|x| I::try_from(x).ok()).map(Self).ok_or(Error::IntegerOverflow)
			}
		})*
	};
}

macro_rules! impl_from_integer {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl<I: IntType> From<Integer<I>> for $larger {
			fn from(int: Integer<I>) -> Self {
				int.0.into() as _
			}
		})*
		$(impl<I: IntType> TryFrom<Integer<I>> for $smaller {
			type Error = Error;

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
