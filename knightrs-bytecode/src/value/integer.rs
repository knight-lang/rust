use crate::parser::Parseable;
use crate::program::Compilable;
use crate::program::Compiler;
use crate::value::{Boolean, KString, List, NamedType, ToBoolean, ToKString, ToList};
use crate::vm::{ParseError, ParseErrorKind, Parseable_OLD, Parser};
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

impl Parseable for Integer {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_>) -> Result<Option<Self::Output>, ParseError> {
		let Some(digits) = parser.take_while(|c| c.is_ascii_digit()) else {
			return Ok(None);
		};

		digits
			.parse::<IntegerInner>()
			.ok()
			.and_then(|int| Integer::new(int, parser.opts()).ok())
			.map(Some)
			.ok_or_else(|| parser.error(ParseErrorKind::IntegerLiteralOverflow))
	}
}

unsafe impl Compilable for Integer {
	fn compile(self, compiler: &mut Compiler) {
		compiler.push_constant(self.into());
	}
}

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
