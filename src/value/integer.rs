// #![warn(missing_docs)]
//! [`Integer`] and related types.

use crate::env::{Environment, Flags};
use crate::parse::{self, Parsable, Parser};
use crate::value::text::{Character, Encoding};
use crate::value::{Boolean, List, NamedType, Text, ToBoolean, ToList, ToText};
use crate::{Error, Result};
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
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

	/// The minimum value.
	pub const MIN: Self = Self(I::MIN);

	/// The maximum value.
	pub const MAX: Self = Self(I::MAX);

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
	/// Any errors [`I::negate`](IntType::negate) returns are bubbled up.
	pub fn negate(self, flags: &Flags) -> Result<Self> {
		self.0.negate(flags).map(Self)
	}

	/// Adds `self` with `augend`.
	///
	/// # Errors
	/// Any errors [`I::add`](IntType::add) returns are bubbled up.
	pub fn add(self, augend: Self, flags: &Flags) -> Result<Self> {
		self.0.add(augend.0, flags).map(Self)
	}

	/// Subtracts `self` by `subtrahend`.
	///
	/// # Errors
	/// Any errors [`I::subtract`](IntType::subtract) returns are bubbled up.
	pub fn subtract(self, subtrahend: Self, flags: &Flags) -> Result<Self> {
		self.0.subtract(subtrahend.0, flags).map(Self)
	}

	/// Multiplies `self` by `multiplier`.
	///
	/// # Errors
	/// Any errors [`I::multiply`](IntType::multiply) returns are bubbled up.
	pub fn multiply(self, multiplier: Self, flags: &Flags) -> Result<Self> {
		self.0.multiply(multiplier.0, flags).map(Self)
	}

	/// Divides `self` by `multiplier`.
	///
	/// # Errors
	/// Returns [`Error::DivisionByZero`] if `divisor` is zero.
	///
	/// Any errors [`I::divide`](IntType::divide) returns are bubbled up.
	pub fn divide(self, divisor: Self, flags: &Flags) -> Result<Self> {
		if divisor.is_zero() {
			return Err(Error::DivisionByZero);
		}

		self.0.divide(divisor.0, flags).map(Self)
	}

	/// Gets the remainder of `self` and `base`.
	///
	/// # Errors
	/// Returns [`Error::DivisionByZero`] if `divisor` is zero.
	///
	/// If [`check_integer_function_bounds`] is enabled and either `self` or `rhs` is negative, an
	/// [`Error::DomainError`] is returned.
	///
	/// Any errors [`I::remainder`](IntType::remainder) returns are bubbled up.
	///
	/// [`check_integer_function_bounds`]: crate::env::flags::Compliance::check_integer_function_bounds
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

		match exponent.cmp(&Self::ZERO) {
			#[cfg(feature = "compliance")]
			Ordering::Less if flags.compliance.check_integer_function_bounds => {
				Err(Error::DomainError("negative exponent"))
			}

			Ordering::Less => match self.0.into() {
				-1 => Ok(if exponent.0.into() % 2 == 0 { self } else { Self::ONE }),
				0 => Err(Error::DivisionByZero),
				1 => Ok(Self::ONE),
				_ => Ok(Self::ZERO),
			},

			Ordering::Equal => Ok(Self::ONE),

			Ordering::Greater => {
				let exp = u32::try_from(exponent).or(Err(Error::DomainError("exponent too large")))?;
				self.0.power(exp, flags).map(Self)
			}
		}
	}

	/// Gets the amount of digits in `self`
	pub fn number_of_digits(self) -> usize {
		// match self.cmp(&Self::ZERO) {
		// 	Ordering::Greater => self.0.log10() as usize,
		// 	Ordering::Equal => 0,
		// 	Ordering::Less => self.0.negate(),
		// }
		self.0.log10() // TODO
	}

	/// Attempts to interpret `self` as an UTF8 codepoint.
	pub fn chr<E: Encoding>(self) -> Result<Character<E>> {
		u32::try_from(self.0.into())
			.ok()
			.and_then(char::from_u32)
			.and_then(Character::new)
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
		rng.gen_range(match () {
			#[cfg(feature = "compliance")]
			_ if flags.compliance.limit_rand_range => Self::ZERO..=0x7FFF.into(),

			#[cfg(feature = "iffy-extensions")]
			_ if flags.extensions.iffy.negative_random_integers => Self::MIN..=Self::MAX,

			_ => Self::ZERO..=Self::MAX,
		})
	}
}

pub struct UniformIntType<I: IntType>(<I as SampleUniform>::Sampler);

impl<I: IntType> SampleUniform for Integer<I> {
	type Sampler = UniformIntType<I>;
}

impl<I: IntType> UniformSampler for UniformIntType<I> {
	type X = Integer<I>;

	fn new<B1, B2>(low: B1, high: B2) -> Self
	where
		B1: SampleBorrow<Self::X>,
		B2: SampleBorrow<Self::X>,
	{
		Self(I::Sampler::new(low.borrow().0, high.borrow().0))
	}

	fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
	where
		B1: SampleBorrow<Self::X>,
		B2: SampleBorrow<Self::X>,
	{
		Self(I::Sampler::new_inclusive(low.borrow().0, high.borrow().0))
	}

	fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
		Integer(self.0.sample(rng))
	}
}

impl<I: IntType, E: Encoding> Parsable<I, E> for Integer<I> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> parse::Result<Option<Self>> {
		parser
			.take_while(Character::is_numeric)
			.map(|src| src.parse())
			.transpose()
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

		let mut integer = self.0.into();
		let mut digits = Vec::with_capacity(self.number_of_digits());

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
	type Err = <I as FromStr>::Err;

	fn from_str(source: &str) -> std::result::Result<Self, Self::Err> {
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

		I::from_str(start).map(Self)
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
