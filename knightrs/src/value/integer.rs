//! [`Integer`] and related types.

use crate::env::{Environment, Flags};
use crate::parse::{self, Parsable, Parser};
use crate::value::{Boolean, List, NamedType, Text, ToBoolean, ToList, ToText};
use crate::{Error, Result};
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

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
pub struct Integer(i64);

/// Represents the ability to be converted to an [`Integer`].
pub trait ToInteger {
	/// Converts `self` to an [`Integer`].
	fn to_integer(&self, env: &mut Environment) -> Result<Integer>;
}

impl PartialEq<i64> for Integer {
	#[inline]
	fn eq(&self, rhs: &i64) -> bool {
		self.0 == *rhs
	}
}

impl PartialOrd<i64> for Integer {
	#[inline]
	fn partial_cmp(&self, rhs: &i64) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(rhs)
	}
}

impl PartialEq<i32> for Integer {
	#[inline]
	fn eq(&self, rhs: &i32) -> bool {
		self.0 == *rhs as i64
	}
}

impl PartialOrd<i32> for Integer {
	#[inline]
	fn partial_cmp(&self, rhs: &i32) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&(*rhs as i64))
	}
}

impl Debug for Integer {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl Display for Integer {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl NamedType for Integer {
	const TYPENAME: &'static str = "Integer";
}

impl Integer {
	/// Creates a new `Integer`.
	#[inline]
	pub const unsafe fn new_unchecked(int: i64) -> Self {
		Self(int)
	}

	/// Creates a new `Integer`.
	#[inline]
	pub const fn new(int: i64, flags: &Flags) -> Option<Self> {
		#[cfg(feature = "compliance")]
		if flags.compliance.i32_integer && (int < Self::min(flags).0 || int > Self::max(flags).0) {
			return None;
		}

		let _ = flags;
		Some(Self(int))
	}

	/// The value zero.
	pub const ZERO: Self = Self(0);

	/// The value one.
	pub const ONE: Self = Self(1);

	#[inline]
	pub const fn max(flags: &Flags) -> Self {
		#[cfg(feature = "compliance")]
		if flags.compliance.i32_integer {
			return Self(i32::MAX as i64);
		}

		Self(i64::MAX)
	}

	#[inline]
	pub const fn min(flags: &Flags) -> Self {
		#[cfg(feature = "compliance")]
		if flags.compliance.i32_integer {
			return Self(i32::MIN as i64);
		}

		Self(i64::MIN)
	}

	/// Negates `self`.
	///
	/// # Errors
	/// Any errors [`::negate`](IntType::negate) returns are bubbled up.
	///
	/// # Examples
	/// ```
	/// # use knightrs::value::Integer;
	/// assert_eq!(1, Integer::new(-1).negate().unwrap());
	/// assert_eq!(-2, Integer::new(2).negate().unwrap());
	/// ```
	pub fn negate(self, flags: &Flags) -> Result<Self> {
		#[allow(unused_mut)]
		let mut opt = Some(self.0.wrapping_neg());

		#[cfg(feature = "compliance")]
		if flags.compliance.check_overflow {
			opt = self.0.checked_neg();
		}

		opt.and_then(|int| Self::new(int, flags)).ok_or(Error::IntegerOverflow)
	}

	fn binary_op<T>(
		self,
		rhs: T,
		flags: &Flags,
		#[allow(unused)] checked: fn(i64, T) -> Option<i64>,
		wrapping: fn(i64, T) -> i64,
	) -> Result<Self> {
		match () {
			#[cfg(feature = "compliance")]
			_ if flags.compliance.check_overflow => checked(self.0, rhs),
			_ => Some(wrapping(self.0, rhs)),
		}
		.and_then(|int| Self::new(int, flags))
		.ok_or(Error::IntegerOverflow)
	}

	/// Adds `self` with `augend`.
	///
	/// # Errors
	/// Any errors [`::add`](IntType::add) returns are bubbled up.
	pub fn add(self, augend: Self, flags: &Flags) -> Result<Self> {
		self.binary_op(augend.0, flags, i64::checked_add, i64::wrapping_add)
	}

	/// Subtracts `self` by `subtrahend`.
	///
	/// # Errors
	/// Any errors [`::subtract`](IntType::subtract) returns are bubbled up.
	pub fn subtract(self, subtrahend: Self, flags: &Flags) -> Result<Self> {
		self.binary_op(subtrahend.0, flags, i64::checked_sub, i64::wrapping_sub)
	}

	/// Multiplies `self` by `multiplier`.
	///
	/// # Errors
	/// Any errors [`::multiply`](IntType::multiply) returns are bubbled up.
	pub fn multiply(self, multiplier: Self, flags: &Flags) -> Result<Self> {
		self.binary_op(multiplier.0, flags, i64::checked_mul, i64::wrapping_mul)
	}

	/// Divides `self` by `multiplier`.
	///
	/// # Errors
	/// Returns [`Error::DivisionByZero`] if `divisor` is zero.
	///
	/// Any errors [`::divide`](IntType::divide) returns are bubbled up.
	pub fn divide(self, divisor: Self, flags: &Flags) -> Result<Self> {
		if divisor == 0 {
			return Err(Error::DivisionByZero);
		}

		self.binary_op(divisor.0, flags, i64::checked_div, i64::wrapping_div)
	}

	/// Gets the remainder of `self` and `base`.
	///
	/// # Errors
	/// Returns [`Error::DivisionByZero`] if `divisor` is zero.
	///
	/// If [`check_integer_function_bounds`] is enabled and either `self` or `rhs` is negative, an
	/// [`Error::DomainError`] is returned.
	///
	/// Any errors [`::remainder`](IntType::remainder) returns are bubbled up.
	///
	/// [`check_integer_function_bounds`]: crate::env::flags::Compliance::check_integer_function_bounds
	pub fn remainder(self, base: Self, flags: &Flags) -> Result<Self> {
		if base == 0 {
			return Err(Error::DivisionByZero);
		}

		#[cfg(feature = "compliance")]
		if flags.compliance.check_integer_function_bounds {
			if self < 0 {
				return Err(Error::DomainError("remainder with a negative number"));
			}

			if base < 0 {
				return Err(Error::DomainError("remainder by a negative base"));
			}
		}

		self.binary_op(base.0, flags, i64::checked_rem, i64::wrapping_rem)
	}

	/// Raises `self` to the `exponent`th power.
	///
	/// # Errors
	/// If the exponent is negative and [`check_integer_function_bounds`] is enabled, then an
	/// [`Error::DomainError`] is returned.
	///
	/// If the exponent is negative, [`check_integer_function_bounds`] isn't enabled, and `self` is
	/// zero, an [`Error::DivisionByZero`] is returned.
	///
	/// If `self` is not zero or one, [`check_integer_function_bounds`] is enabled, and the exponent
	/// is larger than an [`u32`], then an [`Error::DomainError`] is returned.
	///
	/// [`check_integer_function_bounds`]: crate::env::flags::Compliance::check_integer_function_bounds
	/// If the exponent is negative,
	pub fn power(self, exponent: Self, flags: &Flags) -> Result<Self> {
		use std::cmp::Ordering;
		let _ = flags;

		match exponent.cmp(&Self::ZERO) {
			#[cfg(feature = "compliance")]
			Ordering::Less if flags.compliance.check_integer_function_bounds => {
				Err(Error::DomainError("negative exponent"))
			}

			Ordering::Less => match self.0 {
				-1 => Ok(if exponent.0 % 2 == 0 { self } else { Self::ONE }),
				0 => Err(Error::DivisionByZero),
				1 => Ok(Self::ONE),
				_ => Ok(Self::ZERO),
			},

			Ordering::Equal => Ok(Self::ONE),

			Ordering::Greater => {
				let exp = u32::try_from(exponent).or(Err(Error::DomainError("exponent too large")))?;

				self.binary_op(exp, flags, i64::checked_pow, i64::wrapping_pow)
			}
		}
	}

	/// Gets the amount of digits in `self`
	pub fn number_of_digits(self) -> usize {
		match self.cmp(&Self::ZERO) {
			std::cmp::Ordering::Greater => self.0.ilog10() as usize,
			std::cmp::Ordering::Equal => 1,
			std::cmp::Ordering::Less => {
				Self(self.0.checked_neg().unwrap_or(i64::MAX)).number_of_digits()
			}
		}
	}

	/// Attempts to interpret `self` as an UTF8 codepoint.
	pub fn chr(self, flags: &Flags) -> Result<char> {
		u32::try_from(self.0)
			.ok()
			.and_then(char::from_u32)
			.and_then(|c| {
				#[cfg(feature = "compliance")]
				if flags.compliance.knight_encoding && !crate::value::text::is_valid_character(c) {
					return None;
				}

				Some(c)
			})
			.ok_or(Error::DomainError("number isn't a valid char"))
	}

	/// Gets the most significant digit, negating it if `self` is negative.
	#[cfg(feature = "extensions")]
	pub fn head(self) -> Self {
		todo!()

		// let mut n = self.0;
		// while 10 <= n.abs() {
		// 	n /= 10;
		// }
		// Self(n)
	}

	/// Gets everything but the most significant digit.
	#[cfg(feature = "extensions")]
	pub fn tail(self) -> Self {
		// Self(self.0 % 10)
		todo!()
	}

	/// Get a random integer.
	///
	/// # Flags
	/// If the [`limit_rand_range`](crate::env::flags::Compliance::limit_rand_range) flag is enabled,
	/// then the returned integer will be within the range `0..=0x7FFF`.
	///
	/// If the [`negative_random_integers`](crate::env::flags::Iffy::negative_random_integers) flag
	/// is enabled, then the returned integer will be in the range `Self::MIN..=Self::MAX`.
	///
	/// If neither of these flags are enabled, the returned integer will be in the range
	/// `0..Self::MAX`.
	pub fn random<R: rand::Rng + ?Sized>(rng: &mut R, flags: &Flags) -> Self {
		let min = match () {
			#[cfg(feature = "iffy-extensions")]
			_ if flags.extensions.iffy.negative_random_integers => {
				if flags.compliance.i32_integer {
					i32::MIN as i64
				} else {
					i64::MIN
				}
			}

			_ => 0,
		};

		let max = match () {
			#[cfg(feature = "compliance")]
			_ if flags.compliance.limit_rand_range => 0x7FFF,

			#[cfg(feature = "compliance")]
			_ if flags.compliance.i32_integer => i32::MAX as i64,

			_ => i64::MAX,
		};

		let _ = flags;
		Self(rng.gen_range(min..=max))
	}
}

impl Parsable for Integer {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
		parser
			.take_while(|c| c.is_ascii_digit())
			.map(|src| src.parse())
			.transpose()
			.map_err(|_| parser.error(parse::ErrorKind::IntegerLiteralOverflow))
	}
}

impl ToInteger for Integer {
	/// Simply returns `self`.
	#[inline]
	fn to_integer(&self, _: &mut Environment) -> Result<Self> {
		Ok(self.clone())
	}
}

impl ToBoolean for Integer {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_boolean(&self, _: &mut Environment) -> Result<Boolean> {
		Ok(*self != 0)
	}
}

impl ToText for Integer {
	/// Returns a string representation of `self`.
	#[inline]
	fn to_text(&self, _env: &mut Environment) -> Result<Text> {
		// SAFETY: digits are valid in all encodings, and it'll never exceed the length.
		Ok(unsafe { Text::new_unchecked(self) })
	}
}

impl ToList for Integer {
	/// Returns a list of all the digits of `self`, when `self` is expressed in base 10.
	///
	/// If `self` is negative, all the returned digits are negative.
	fn to_list(&self, _: &mut Environment) -> Result<List> {
		if *self == 0 {
			return Ok(List::boxed(self.clone().into()));
		}

		let mut integer = self.0;
		let mut digits = Vec::with_capacity(self.number_of_digits());

		while integer != 0 {
			digits.insert(0, Self(integer % 10).into());
			integer /= 10;
		}

		// The maximum amount of digits for an Integer is vastly smaller than `i32::MAX`, so
		// there's no need to do a check.
		Ok(unsafe { List::new_unchecked(digits) })
	}
}

impl FromStr for Integer {
	type Err = <i64 as FromStr>::Err;

	fn from_str(source: &str) -> std::result::Result<Self, Self::Err> {
		let source = source.trim_start();

		let mut chars = source.chars();
		let mut start = match chars.next() {
			None => return Ok(Self::default()),
			Some('+' | '-') => chars.as_str(),
			_ => source,
		};

		if let Some(bad) = start.find(|c: char| !c.is_ascii_digit()) {
			start = &source[..bad + (start != source) as usize];
		} else if start != source {
			start = source;
		}

		i64::from_str(start).map(Self)
	}
}

macro_rules! impl_integer_from {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl From<$smaller> for Integer {
			fn from(num: $smaller) -> Self {
				Self(i64::from(num as i32))
			}
		})*
		$(impl TryFrom<$larger> for Integer {
			type Error = Error;

			fn try_from(num: $larger) -> Result<Self> {
				i64::try_from(num).ok().and_then(|x| i64::try_from(x).ok()).map(Self).ok_or(Error::IntegerOverflow)
			}
		})*
	};
}

macro_rules! impl_from_integer {
	($($smaller:ident)* ; $($larger:ident)*) => {
		$(impl From<Integer> for $larger {
			fn from(int: Integer) -> Self {
				int.0 as _
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
		char::from_u32(u32::try_from(int)?).ok_or(Error::DomainError("integer isn't a char"))
	}
}
