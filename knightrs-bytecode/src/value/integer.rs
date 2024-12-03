use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::{Character, Encoding};
use crate::value::{Boolean, KString, List, NamedType, ToBoolean, ToKString, ToList};
use crate::{Environment, Error, Options};
use std::fmt::{self, Debug, Display, Formatter};

/// Integer is the integer type within Knight programs
///
/// It's internally always represented by an `i64`. However, constructing new [`Integer`]s via
/// the [`new`] function requires passing in an [`Option`], which allows us to do `i32`-bound
/// checking for compliance.
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
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl Display for Integer {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl PartialEq<IntegerInner> for Integer {
	#[inline]
	fn eq(&self, rhs: &IntegerInner) -> bool {
		self.0 == *rhs
	}
}

impl PartialOrd<IntegerInner> for Integer {
	#[inline]
	fn partial_cmp(&self, rhs: &IntegerInner) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(rhs)
	}
}

/// Problems that can occur when performing operations on integers.
#[derive(Error, Debug)]
pub enum IntegerError {
	/// A number doesn't fit in an `i32`; Only used when `compliance.i32_integer` is enabled.
	#[deprecated]
	#[cfg(feature = "compliance")]
	#[error("integer {0} doesn't fit in an i32")]
	IntegerOutOfBounds(IntegerInner),

	#[cfg(feature = "compliance")]
	#[error("method {0:?} overflowed the bounds")]
	MethodOverflow(char),

	/// Division/Remainder/Power by zero.
	#[error("{0}")]
	DivisionByZero(ZeroDivisionKind),

	/// The arguments passed to a function are outside of its expected domain.
	#[error("domain error: {0}")]
	DomainError(&'static str),

	/// Means `chr` was called on an int and it's not valid for an encoding.
	#[error("integer {0:?} isn't a valid char for {1:?}")]
	NotAValidChar(Integer, Encoding),
}

/// Helper type fir [`IntegerError::DivisionByZero`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZeroDivisionKind {
	/// A `/` by zero occurred.
	Divide,
	/// A `%` by zero occurred.
	Remainder,
	/// A `^` with the first arg as 0 and the second is a negative number.
	Power,
}

impl Display for ZeroDivisionKind {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Divide => f.write_str("division by zero"),
			Self::Remainder => f.write_str("remainder by zero"),
			Self::Power => f.write_str("0 exponentiated by a negative power"),
		}
	}
}

impl Integer {
	/// The value zero.
	pub const ZERO: Self = Self(0);

	/// The value one.
	pub const ONE: Self = Self(1);

	/// Returns the value contained within the integer.
	pub const fn inner(self) -> IntegerInner {
		self.0
	}

	/// Creates a new [`Integer`] without doing compliance validations.
	///
	/// # Compliance
	/// This should only be called with `int`s known to be within the range of `i32`s; Calling it
	/// with integers outside `i32`'s bounds can make non-spec-compliant programs seem compliant.
	#[inline]
	pub const fn new_unvalidated(int: IntegerInner) -> Self {
		// Sanity check, make sure it's in bounds.
		debug_assert!((i32::MIN as IntegerInner) <= int && int <= (i32::MAX as IntegerInner));

		Self(int)
	}

	#[inline]
	#[doc(hidden)]
	pub const fn new_unvalidated_unchecked(int: IntegerInner) -> Self {
		Self(int)
	}

	/// Tries to create a new [`Integer`], with the given options.
	///
	/// Without `compliance.i32_integer` enabled, this function never fails. When it's enabled, an
	/// [`IntegerError::IntegerOutOfBounds`] is returned if the integer is not within the bounds.
	#[cfg_attr(not(feature = "compliance"), inline)]
	#[deprecated(note = "use `new` as it returns `Option` and the caller can do errors themselves")]
	pub fn new_error(int: IntegerInner, opts: &Options) -> Result<Self, IntegerError> {
		#[cfg(feature = "compliance")]
		if opts.compliance.i32_integer && !(Self::min(opts).0..Self::max(opts).0).contains(&int) {
			return Err(IntegerError::IntegerOutOfBounds(int));
		}

		Ok(Self(int))
	}

	/// Tries to create a new [`Integer`], with the given options.
	///
	/// When `compliance.i32_integer` is enabled, this function will return `None` if the given `int`
	/// is not within the bounds of an `i32` integer. When it's disabled, this function never fails.
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn new(int: IntegerInner, opts: &Options) -> Option<Self> {
		#[cfg(feature = "compliance")]
		if opts.compliance.i32_integer && !(Self::min(opts).0..Self::max(opts).0).contains(&int) {
			return None;
		}

		Some(Self(int))
	}

	/// Returns the maximum value for [`Integer`]s given `opts`.
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn max(opts: &Options) -> Self {
		#[cfg(feature = "compliance")]
		if opts.compliance.i32_integer {
			return Self(i32::MAX as IntegerInner);
		}

		Self(IntegerInner::MAX)
	}

	/// Returns the minimum value for [`Integer`]s given `opts`.
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn min(opts: &Options) -> Self {
		#[cfg(feature = "compliance")]
		if opts.compliance.i32_integer {
			return Self(i32::MIN as IntegerInner);
		}

		Self(IntegerInner::MIN)
	}

	/// Negates `self`, wrapping unless `opts.compliance.check_overflow` is on.
	///
	/// # Errors
	/// If `opts.compliance.check_overflow` is on, overflows yield [`IntegerError::MethodOverflow`].
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn negate(self, opts: &Options) -> Result<Self, IntegerError> {
		#[cfg(feature = "compliance")]
		{
			match () {
				#[cfg(feature = "compliance")]
				_ if opts.compliance.check_overflow => self.0.checked_neg(),
				_ => Some(self.0.wrapping_neg()),
			}
			.and_then(|int| Self::new(int, opts))
			.ok_or(IntegerError::MethodOverflow('~'))
		}

		#[cfg(not(feature = "compliance"))]
		{
			Ok(Self::new_unvalidated_unchecked(self.0.wrapping_neg()))
		}
	}

	fn binary_op<T>(
		self,
		rhs: T,
		opts: &Options,
		func: char,
		#[allow(unused)] checked: fn(i64, T) -> Option<i64>,
		wrapping: fn(i64, T) -> i64,
	) -> Result<Self, IntegerError> {
		#[cfg(feature = "compliance")]
		{
			match () {
				#[cfg(feature = "compliance")]
				_ if opts.compliance.check_overflow => checked(self.0, rhs),
				_ => Some(wrapping(self.0, rhs)),
			}
			.and_then(|int| Self::new(int, opts))
			.ok_or(IntegerError::MethodOverflow(func))
		}

		#[cfg(not(feature = "compliance"))]
		{
			Ok(Self::new_unvalidated_unchecked(wrapping(self.0, rhs)))
		}
	}

	/// Adds `augend` to `self`, wrapping unless `opts.compliance.check_overflow` is on.
	///
	/// # Errors
	/// If `opts.compliance.check_overflow` is on, overflows yield [`IntegerError::MethodOverflow`].
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn add(self, augend: Self, opts: &Options) -> Result<Self, IntegerError> {
		self.binary_op(augend.0, opts, '+', i64::checked_add, i64::wrapping_add)
	}

	/// Subtracts `subtrahend` from `self`, wrapping unless `opts.compliance.check_overflow` is on.
	///
	/// # Errors
	/// If `opts.compliance.check_overflow` is on, overflows yield [`IntegerError::MethodOverflow`].
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn subtract(self, subtrahend: Self, opts: &Options) -> Result<Self, IntegerError> {
		self.binary_op(subtrahend.0, opts, '-', i64::checked_sub, i64::wrapping_sub)
	}

	/// Multiplies `self` by `multiplier`, wrapping unless `opts.compliance.check_overflow` is on.
	///
	/// # Errors
	/// If `opts.compliance.check_overflow` is on, overflows yield [`IntegerError::MethodOverflow`].
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn multiply(self, multiplier: Self, opts: &Options) -> Result<Self, IntegerError> {
		self.binary_op(multiplier.0, opts, '*', i64::checked_mul, i64::wrapping_mul)
	}

	/// Divides `self` by `divisor`, wrapping unless `opts.compliance.check_overflow` is on.
	///
	/// # Errors
	/// If `divisor` is zero, an [`IntegerError::DivisionByZero`] is returned.
	///
	/// If `opts.compliance.check_overflow` is on, overflows yield [`IntegerError::MethodOverflow`].
	pub fn divide(self, divisor: Self, opts: &Options) -> Result<Self, IntegerError> {
		if divisor == 0 {
			return Err(IntegerError::DivisionByZero(ZeroDivisionKind::Divide));
		}

		self.binary_op(divisor.0, opts, '/', i64::checked_div, i64::wrapping_div)
	}

	/// Gets the remainder of `self / base`, wrapping unless `opts.compliance.check_overflow` is on.
	///
	/// # Errors
	/// If `base` is zero, an [`IntegerError::DivisionByZero`] is returned.
	///
	/// If `compliance.check_integer_function_bounds` is enabled, then a [`DomainError`] is returned
	/// when either `self` or `base` are negative.
	///
	/// If `opts.compliance.check_overflow` is on, overflows yield [`IntegerError::MethodOverflow`].
	pub fn remainder(self, base: Self, opts: &Options) -> Result<Self, IntegerError> {
		if base == 0 {
			return Err(IntegerError::DivisionByZero(ZeroDivisionKind::Remainder));
		}

		#[cfg(feature = "compliance")]
		if opts.compliance.check_integer_function_bounds {
			if self < 0 {
				return Err(IntegerError::DomainError("remainder with a negative number"));
			}

			if base < 0 {
				return Err(IntegerError::DomainError("remainder by a negative base"));
			}
		}

		self.binary_op(base.0, opts, '%', i64::checked_rem, i64::wrapping_rem)
	}

	/// Raises `self` to `exponent`, wrapping unless `opts.compliance.check_overflow` is on.
	///
	/// # Errors
	/// If the exponent is negative and `compliance.check_integer_function_bounds` is enabled, then
	/// an [`Error::DomainError`] is returned.
	///
	/// If the exponent is negative, `compliance.check_integer_function_bounds` isn't enabled, and
	/// `self` is zero, an [`Error::DivisionByZero`] is returned.
	///
	/// If `self` is not zero or one, `compliance.check_integer_function_bounds` is enabled, and the
	/// exponent is larger than an [`u32`], then an [`Error::DomainError`] is returned.
	///
	/// If `opts.compliance.check_overflow` is on, overflows yield [`IntegerError::MethodOverflow`].
	pub fn power(self, exponent: Self, opts: &Options) -> Result<Self, IntegerError> {
		use std::cmp::Ordering;

		// We do different things based on the exponent
		match exponent.cmp(&Self::ZERO) {
			Ordering::Less => match self.0 {
				// When `check_integer_function_bounds` is enabled, don't allow negative exponents.
				#[cfg(feature = "compliance")]
				_ if opts.compliance.check_integer_function_bounds => {
					Err(IntegerError::DomainError("negative exponent"))
				}

				// Special cases for negative exponents of -1, 0, and 1.
				-1 => Ok(if exponent.0 % 2 == 0 { self } else { Self::ONE }),
				0 => Err(IntegerError::DivisionByZero(ZeroDivisionKind::Power)),
				1 => Ok(Self::ONE),

				// Otherwise, return 0, as everything else is below zero.
				_ => Ok(Self::ZERO),
			},

			// Anything to the `0`th power is one.
			Ordering::Equal => Ok(Self::ONE),

			// Positive exponents
			Ordering::Greater => match u32::try_from(exponent.inner()) {
				// If the exponent could fit in a `u32`, then perform the normal operation
				Ok(exp) => self.binary_op(exp, opts, '^', i64::checked_pow, i64::wrapping_pow),

				// It was too large to fit in a `u32`; special-case `0` and `1` which won't overflow
				Err(_) => match self.inner() {
					// 0 to massive integers is zero
					0 | 1 => Ok(self),

					// Anything else means the exponent was far too large.
					_ => Err(IntegerError::DomainError("exponent too large")),
				},
			},
		}
	}

	/// Gets the amount of digits in `self`.
	pub fn number_of_digits(self) -> usize {
		use std::cmp::Ordering;

		match self.cmp(&Self::ZERO) {
			Ordering::Greater => self.0.ilog10() as usize + 1,
			Ordering::Equal => 1,
			Ordering::Less => Self(self.0.checked_neg().unwrap_or(i64::MAX)).number_of_digits(),
		}
	}

	/// Attempts to interpret `self` as a char from the given encoding.
	pub fn chr(self, opts: &Options) -> Result<Character, IntegerError> {
		u32::try_from(self.0)
			.ok()
			.and_then(char::from_u32)
			.and_then(|chr| Character::new(chr, &opts.encoding))
			.ok_or(IntegerError::NotAValidChar(self, opts.encoding))
	}

	/// Parses out an integer from `source` according to the Knight specifications for string ->
	/// integer conversions.
	///
	/// This could certainly be cleaned up.
	pub fn parse_from_str(source: &str, opts: &Options) -> crate::Result<Self> {
		// This could definitely be cleaned up
		let source = source.trim_start();

		let mut chars = source.chars();
		let mut start = match chars.next() {
			None => return Ok(Self::ZERO),
			Some('+' | '-') => chars.as_str(),
			_ => source,
		};

		if let Some(bad) = start.find(|c: char| !c.is_ascii_digit()) {
			start = &source[..bad + (start != source) as usize];
		} else if start != source {
			start = source;
		}

		match <IntegerInner as std::str::FromStr>::from_str(start) {
			Ok(value) => Ok(Self::new_error(value, opts)?),
			Err(err) => match err.kind() {
				std::num::IntErrorKind::Empty | std::num::IntErrorKind::InvalidDigit => Ok(Self::ZERO),
				std::num::IntErrorKind::PosOverflow | std::num::IntErrorKind::NegOverflow => {
					todo!("integer overflow error")
				}
				other => todo!("{:?}", other),
			},
		}
	}
}

impl Parseable for Integer {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, '_>) -> Result<Option<Self::Output>, ParseError> {
		let Some(digits) = parser.take_while(|c| c.is_ascii_digit()) else {
			return Ok(None);
		};

		#[cfg(feature = "extensions")]
		if parser.peek().map_or(false, |c| c == '.') {
			todo!("float extensions. (really this should be its own `Parseable`");
		}

		digits
			.parse::<IntegerInner>()
			.ok()
			.and_then(|int| Integer::new(int, parser.opts()))
			.map(Some)
			.ok_or_else(|| parser.error(ParseErrorKind::IntegerLiteralOverflow))
	}
}

unsafe impl<'path> Compilable<'path> for Integer {
	fn compile(self, compiler: &mut Compiler, _: &Options) -> Result<(), ParseError> {
		compiler.push_constant(self.into());
		Ok(())
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
		Ok(*self != 0)
	}
}

impl ToKString for Integer {
	/// Returns whether `self` is nonzero.
	#[inline]
	fn to_kstring(&self, _: &mut Environment) -> crate::Result<KString> {
		// COMPLIANCE: `Integer#to_string` yields just an optional leading `-` followed by digits,
		// which is valid in all encodings. Additionally, it's nowhere near the maximum length for a
		// string.
		Ok(KString::new_unvalidated(self.to_string()))
	}
}

impl ToList for Integer {
	fn to_list(&self, env: &mut Environment) -> crate::Result<List> {
		#[cfg(feature = "compliance")]
		if env.opts().compliance.disallow_negative_int_to_list && *self < 0 {
			return Err(Error::DomainError("negative integer for to list encountered"));
		}

		if *self == 0 {
			return Ok(List::boxed((*self).into()));
		}

		let mut integer = self.0;
		let mut digits = Vec::with_capacity(self.number_of_digits());

		while integer != 0 {
			digits.insert(0, Self(integer % 10).into());
			integer /= 10;
		}

		// COMPLIANCE: The maximum amount of digits in an integer is vastly smaller than the maximum
		// size of `i32::MAX`.
		Ok(List::new_unvalidated(digits))
	}
}
