use crate::old_vm_and_parser_and_program::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::value::{Boolean, KString, List, NamedType, ToBoolean, ToKString, ToList};
use crate::{options::Options, Environment};
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer(IntegerInner);

type IntegerInner = i64;

/// Represents the ability to be converted to an [`Integer`].
pub trait ToInteger {
	/// Converts `self` to an [`Integer`].
	fn to_integer(&self, env: &mut Environment) -> crate::Result<Integer>;
}

impl NamedType for Integer {
	#[inline]
	fn type_name(&self) -> &'static str {
		"Integer"
	}
}
impl Debug for Integer {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl Display for Integer {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

#[derive(Error, Debug)]
pub enum IntegerError {
	#[cfg(feature = "compliance")]
	#[error("{0} is out of bounds for integers")]
	OutOfBounds(IntegerInner),

	#[cfg(feature = "compliance")]
	#[error("overflow for method {0:?}")]
	Overflow(char),

	#[error("division by zero for {0:?}")]
	DivisionByZero(char),

	#[error("domain error: {0}")]
	DomainError(&'static str),
}

impl Integer {
	/// The value zero.
	pub const ZERO: Self = Self(0);

	/// The value one.
	pub const ONE: Self = Self(1);

	pub const fn inner(self) -> IntegerInner {
		self.0
	}

	#[inline]
	pub const fn new_unvalidated(int: IntegerInner) -> Self {
		Self(int)
	}

	// TODO: return just an `Option`, so the caller can deal/construct `OutOufBounds` itself.
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn new(int: IntegerInner, opts: &Options) -> Result<Self, IntegerError> {
		#[cfg(feature = "compliance")]
		if opts.compliance.i32_integer && !(Self::min(opts).0..Self::max(opts).0).contains(&int) {
			return Err(IntegerError::OutOfBounds(int));
		}

		Ok(Self(int))
	}

	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn max(opts: &Options) -> Self {
		#[cfg(feature = "compliance")]
		if opts.compliance.i32_integer {
			return Self(i32::MAX as IntegerInner);
		}

		Self(IntegerInner::MAX)
	}

	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn min(opts: &Options) -> Self {
		#[cfg(feature = "compliance")]
		if opts.compliance.i32_integer {
			return Self(i32::MIN as IntegerInner);
		}

		Self(IntegerInner::MIN)
	}

	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn negate(self, opts: &Options) -> Result<Self, IntegerError> {
		let value = match () {
			#[cfg(feature = "compliance")]
			_ if opts.compliance.check_overflow => {
				self.0.checked_neg().ok_or(IntegerError::Overflow('-'))?
			}
			_ => self.0.wrapping_neg(),
		};

		Self::new(value, opts)
	}

	fn binary_op<T>(
		self,
		rhs: T,
		opts: &Options,
		func: char,
		#[allow(unused)] checked: fn(i64, T) -> Option<i64>,
		wrapping: fn(i64, T) -> i64,
	) -> Result<Self, IntegerError> {
		let value = match () {
			#[cfg(feature = "compliance")]
			_ if opts.compliance.check_overflow => {
				checked(self.0, rhs).ok_or(IntegerError::Overflow(func))?
			}
			_ => wrapping(self.0, rhs),
		};

		Self::new(value, opts)
	}

	pub fn add(self, augend: Self, opts: &Options) -> Result<Self, IntegerError> {
		self.binary_op(augend.0, opts, '+', i64::checked_add, i64::wrapping_add)
	}

	pub fn subtract(self, subtrahend: Self, opts: &Options) -> Result<Self, IntegerError> {
		self.binary_op(subtrahend.0, opts, '-', i64::checked_sub, i64::wrapping_sub)
	}

	pub fn multiply(self, multiplier: Self, opts: &Options) -> Result<Self, IntegerError> {
		self.binary_op(multiplier.0, opts, '*', i64::checked_mul, i64::wrapping_mul)
	}

	pub fn divide(self, divisor: Self, opts: &Options) -> Result<Self, IntegerError> {
		if divisor == Self::ZERO {
			return Err(IntegerError::DivisionByZero('/'));
		}

		self.binary_op(divisor.0, opts, '/', i64::checked_div, i64::wrapping_div)
	}

	pub fn remainder(self, base: Self, opts: &Options) -> Result<Self, IntegerError> {
		if base == Self::ZERO {
			return Err(IntegerError::DivisionByZero('%'));
		}

		#[cfg(feature = "compliance")]
		if opts.compliance.check_integer_function_bounds {
			if self < Self::ZERO {
				return Err(IntegerError::DomainError("remainder with a negative number"));
			}

			if base < Self::ZERO {
				return Err(IntegerError::DomainError("remainder by a negative base"));
			}
		}

		self.binary_op(base.0, opts, '%', i64::checked_rem, i64::wrapping_rem)
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
	/// [`check_integer_function_bounds`]: crate::env::opts::Compliance::check_integer_function_bounds
	/// If the exponent is negative,
	pub fn power(self, exponent: Self, opts: &Options) -> Result<Self, IntegerError> {
		use std::cmp::Ordering;
		let _ = opts;

		match exponent.cmp(&Self::ZERO) {
			#[cfg(feature = "compliance")]
			Ordering::Less if opts.compliance.check_integer_function_bounds => {
				Err(IntegerError::DomainError("negative exponent"))
			}

			Ordering::Less => match self.0 {
				-1 => Ok(if exponent.0 % 2 == 0 { self } else { Self::ONE }),
				0 => Err(IntegerError::DivisionByZero('^')),
				1 => Ok(Self::ONE),
				_ => Ok(Self::ZERO),
			},

			Ordering::Equal => Ok(Self::ONE),

			Ordering::Greater => {
				let exp = u32::try_from(exponent.inner())
					.or(Err(IntegerError::DomainError("exponent too large")))?;

				self.binary_op(exp, opts, '^', i64::checked_pow, i64::wrapping_pow)
			}
		}
	}

	/// Gets the amount of digits in `self`
	pub fn number_of_digits(self) -> usize {
		match self.cmp(&Self::ZERO) {
			std::cmp::Ordering::Greater => self.0.ilog10() as usize + 1,
			std::cmp::Ordering::Equal => 1,
			std::cmp::Ordering::Less => {
				Self(self.0.checked_neg().unwrap_or(i64::MAX)).number_of_digits()
			}
		}
	}

	/// Attempts to interpret `self` as a char in the given encoding.
	pub fn chr(self, opts: &Options) -> Result<char, IntegerError> {
		u32::try_from(self.0)
			.ok()
			.and_then(char::from_u32)
			.and_then(|chr| opts.encoding.is_char_valid(chr).then_some(chr))
			.ok_or(IntegerError::DomainError("number isn't a valid char"))
	}
}

unsafe impl Parseable for Integer {
	fn parse(parser: &mut Parser<'_, '_>) -> Result<bool, ParseError> {
		let Some(digits) = parser.take_while(|c| c.is_ascii_digit()) else {
			return Ok(false);
		};

		match digits
			.parse::<IntegerInner>()
			.ok()
			.and_then(|int| Integer::new(int, parser.opts()).ok())
		{
			Some(integer) => {
				parser.builder().push_constant(integer.into());
				Ok(true)
			}
			None => Err(parser.error(ParseErrorKind::IntegerLiteralOverflow)),
		}
	}
}

// impl Parsable for Integer {
// 	type Output = Self;

// 	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
// 		parser
// 			.take_while(|c| c.is_ascii_digit())
// 			.map(|src| src.parse())
// 			.transpose()
// 			.map_err(|_| parser.error(parse::ErrorKind::IntegerLiteralOverflow))
// 	}
// }

impl ToInteger for Integer {
	/// Simply returns `self`.
	#[inline]
	fn to_integer(&self, _: &mut Environment) -> crate::Result<Self> {
		Ok(*self)
	}
}

impl ToBoolean for Integer {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_boolean(&self, _: &mut Environment) -> crate::Result<Boolean> {
		Ok(*self != Self::ZERO)
	}
}

impl ToKString for Integer {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_kstring(&self, _: &mut Environment) -> crate::Result<KString> {
		// Note: number -> string conversions are valid in _all_ environments,
		// so there's no need to check: digits are in bounds, and correct encodings.
		Ok(KString::new_unvalidated(&self.to_string()))
	}
}

impl ToList for Integer {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_list(&self, _: &mut Environment) -> crate::Result<List> {
		// Ok(*self != Self::ZERO)
		todo!()
	}
}

// impl ToText for Integer {
// 	/// Returns a string representation of `self`.
// 	#[inline]
// 	fn to_text(&self, _env: &mut Environment) -> Result<Text> {
// 		// SAFETY: digits are valid in all encodings, and it'll never exceed the length.
// 		Ok(unsafe { Text::new_unchecked(self) })
// 	}
// }

// impl ToList for Integer {
// 	/// Returns a list of all the digits of `self`, when `self` is expressed in base 10.
// 	///
// 	/// If `self` is negative, all the returned digits are negative.
// 	fn to_list(&self, _: &mut Environment) -> Result<List> {
// 		if *self == 0 {
// 			return Ok(List::boxed(self.clone().into()));
// 		}

// 		let mut integer = self.0;
// 		let mut digits = Vec::with_capacity(self.number_of_digits());

// 		while integer != 0 {
// 			digits.insert(0, Self(integer % 10).into());
// 			integer /= 10;
// 		}

// 		// The maximum amount of digits for an Integer is vastly smaller than `i32::MAX`, so
// 		// there's no need to do a check.
// 		Ok(unsafe { List::new_unchecked(digits) })
// 	}
// }

// impl FromStr for Integer {
// 	type Err = <i64 as FromStr>::Err;

// 	fn from_str(source: &str) -> std::result::Result<Self, Self::Err> {
// 		let source = source.trim_start();

// 		let mut chars = source.chars();
// 		let mut start = match chars.next() {
// 			None => return Ok(Self::default()),
// 			Some('+' | '-') => chars.as_str(),
// 			_ => source,
// 		};

// 		if let Some(bad) = start.find(|c: char| !c.is_ascii_digit()) {
// 			start = &source[..bad + (start != source) as usize];
// 		} else if start != source {
// 			start = source;
// 		}

// 		i64::from_str(start).map(Self)
// 	}
// }

// macro_rules! impl_integer_from {
// 	($($smaller:ident)* ; $($larger:ident)*) => {
// 		$(impl From<$smaller> for Integer {
// 			#[inline]
// 			fn from(num: $smaller) -> Self {
// 				Self(i64::from(num as i32))
// 			}
// 		})*
// 		$(impl TryFrom<$larger> for Integer {
// 			type Error = Error;

// 			#[inline]
// 			fn try_from(num: $larger) -> Result<Self, Error> {
// 				i64::try_from(num).ok().and_then(|x| i64::try_from(x).ok()).map(Self).ok_or(Error::Overflow)
// 			}
// 		})*
// 	};
// }

// macro_rules! impl_from_integer {
// 	($($smaller:ident)* ; $($larger:ident)*) => {
// 		$(impl From<Integer> for $larger {
// 			fn from(int: Integer) -> Self {
// 				int.0 as _
// 			}
// 		})*
// 		$(impl TryFrom<Integer> for $smaller {
// 			type Error = Error;

// 			fn try_from(int: Integer) -> Result<Self, Error> {
// 				int.0.try_into().or(Err(Error::Overflow))
// 			}
// 		})*
// 	};
// }

// impl_integer_from!(bool u8 u16 i8 i16 i32 ; u32 u64 u128 usize i64 i128 isize );
// impl_from_integer!(u8 u16 u32 u64 u128 usize i8 i16 i32 isize; i64 i128);

// impl TryFrom<char> for Integer {
// 	type Error = Error;

// 	fn try_from(chr: char) -> Result<Self, Error> {
// 		(chr as u32).try_into()
// 	}
// }

// impl TryFrom<Integer> for char {
// 	type Error = Error;

// 	fn try_from(int: Integer) -> Result<Self, Error> {
// 		char::from_u32(u32::try_from(int)?).ok_or(Error::DomainError("integer isn't a char"))
// 	}
// }
