use crate::value::TAG_SHIFT;
use crate::text::{ToText, Text, TextCow};
use crate::boolean::{ToBoolean, Boolean};
use std::fmt::{self, Display, Formatter};
use crate::ops::*;

cfg_if! {
	if #[cfg(feature="strict-numbers")] {
		/// The number type within Knight.
		pub type NumberType = i32;
		pub type UNumberType = u32;
	} else {
		/// The number type within Knight.
		pub type NumberType = i64;
		pub type UNumberType = u64;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(NumberType);

pub trait ToNumber {
	fn to_number(&self) -> crate::Result<Number>;
}

const fn truncate(num: NumberType) -> NumberType {
	(num << TAG_SHIFT) >> TAG_SHIFT
}

const_assert!(Number::new(Number::MAX.get()).is_some());
const_assert!(Number::new(Number::MIN.get()).is_some());
const_assert!(Number::new(Number::MAX.get() + 1).is_none());
const_assert!(Number::new(Number::MIN.get() - 1).is_none());

impl Number {
	pub const MAX: Self = unsafe { Self::new_unchecked(((UNumberType::MAX) >> (TAG_SHIFT + 1)) as NumberType) };
	pub const MIN: Self = unsafe { Self::new_unchecked(!Self::MAX.0) };
	pub const ZERO: Self = unsafe { Self::new_unchecked(0) };
	pub const ONE: Self = unsafe { Self::new_unchecked(1) };
	pub const NEG_ONE: Self = unsafe { Self::new_unchecked(-1) };

	pub const fn is_valid(num: NumberType) -> bool {
		truncate(num) == num
	}

	pub const fn new(num: NumberType) -> Option<Self> {
		if Self::is_valid(num) {
			Some(Self(num))
		} else {
			None
		}
	}

	pub const unsafe fn new_unchecked(num: NumberType) -> Self {
		debug_assert_const!(Self::is_valid(num));

		Self(num)
	}

	pub const fn new_truncate(num: NumberType) -> Self {
		Self(truncate(num))
	}

	#[deprecated]
	pub const fn inner(self) -> NumberType {
		self.0
	}

	pub const fn get(self) -> NumberType {
		self.0
	}
}

impl ToNumber for Number {
	fn to_number(&self) -> crate::Result<Number> {
		Ok(*self)
	}
}

impl ToBoolean for Number {
	fn to_boolean(&self) -> crate::Result<Boolean> {
		Ok(self.get() != 0)
	}
}

impl ToText<'_, 'static> for Number {
	fn to_text(&self) -> crate::Result<TextCow<'static>> {
		Ok(Text::new(&self.to_string()).unwrap().into())
	}
}

impl Display for Number {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.get(), f)
	}
}

#[derive(Debug)]
pub enum MathError {
	Overflow { func: char, lhs: Number, rhs: Number },
	DivisionByZero { kind: &'static str },
	NegativeModulo { operand: Number },
}

impl Display for MathError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Overflow { func, lhs, rhs } => write!(f, "operation '{} {} {}' overflowed", func, lhs, rhs),
			Self::DivisionByZero { kind } => write!(f, "invalid zero {}", kind),
			Self::NegativeModulo { operand } => write!(f, "cannot modulo with negative numbers ({} invalid)", operand)
		}
	}
}

impl std::error::Error for MathError {}


impl TryAdd for Number {
	type Error = MathError;
	type Output = Self;

	#[cfg(not(feature="checked-overflow"))]
	#[inline]
	fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
		Ok(Self::new_truncate(self.get().wrapping_add(rhs.get())))
	}

	#[cfg(Feature="checked-overflow")]
	fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
		if let Some(sum) = self.get().checked_add(rhs.get()) {
			if let Some(sum) = Self::new(sum) {
				return Ok(sum)
			}
		}

		Err(MathError::Overflow { func: '+', lhs: self, rhs })
	}
}

impl TrySub for Number {
	type Error = MathError;
	type Output = Self;

	#[cfg(not(feature="checked-overflow"))]
	#[inline]
	fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
		Ok(Self::new_truncate(self.get().wrapping_sub(rhs.get())))
	}

	#[cfg(Feature="checked-overflow")]
	fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
		if let Some(sum) = self.get().checked_sub(rhs.get()) {
			if let Some(sum) = Self::new(sum) {
				return Ok(sum)
			}
		}

		Err(MathError::Overflow { func: '-', lhs: self, rhs })
	}
}


impl TryMul for Number {
	type Error = MathError;
	type Output = Self;

	#[cfg(not(feature="checked-overflow"))]
	#[inline]
	fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
		Ok(Self::new_truncate(self.get().wrapping_mul(rhs.get())))
	}

	#[cfg(Feature="checked-overflow")]
	fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
		if let Some(sum) = self.get().checked_mul(rhs.get()) {
			if let Some(sum) = Self::new(sum) {
				return Ok(sum)
			}
		}

		Err(MathError::Overflow { func: '*', lhs: self, rhs })
	}
}

impl TryDiv for Number {
	type Error = MathError;
	type Output = Self;

	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_div(self, rhs: Self) -> Result<Self, Self::Error> {
		let quot = self.get().checked_div(rhs.get()).ok_or(MathError::DivisionByZero { kind: "division" })?;

		cfg_if! {
			if #[cfg(feature="checked-overflow")] {
				Self::new(quot).ok_or(Err(MathError::Overflow { func: '/', lhs: self, rhs }))
			} else {
				Ok(Self::new_truncate(quot))
			}
		}
	}
}

impl TryRem for Number {
	type Error = MathError;
	type Output = Self;

	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_rem(self, rhs: Self) -> Result<Self, Self::Error> {

		if rhs.get() == 0 {
			Err(MathError::DivisionByZero { kind: "moudlo" })
		} else if self.get() < 0 {
			Err(MathError::NegativeModulo { operand: self })
		} else if rhs.get() < 0 {
			Err(MathError::NegativeModulo { operand: rhs })
		} else {
			// it's impossible for a remainder to overflow if both operands are positive.
			unsafe {
				Ok(Self::new_unchecked(self.get() % rhs.get()))
			}
		}
	}
}

impl TryPow for Number {
	type Error = MathError;
	type Output = Self;

	#[cfg_attr(not(feature="checked-overflow"), inline)]
	fn try_pow(self, rhs: Self) -> Result<Self, Self::Error> {
		let lnum = self.get();
		let rnum = rhs.get();

		if rnum < 0 {
			return match lnum {
				0 => Err(MathError::DivisionByZero { kind: "exponentiation by a negative number" }),
				-1 => Ok(if rnum & 1 == 0 { Self::ONE } else { Self::NEG_ONE }),
				1 => Ok(Self::ONE),
				_ => Ok(Self::ZERO)
			};
		}

		if cfg!(not(feature="checked-overflow")) {
			// note that all `i32`s exponentiations can be represented by `f64.powf()`.
			// however, currently, `f64::powf` is not const-stable, so we have to use i32.
			// when it is, use this:
			// 	return Ok(Self::new_truncate((lnum as f64).powf(rnum as f64) as NumberType));
			return Ok(Self::new_truncate(lnum.pow(rnum as u32)));
		}

		debug_assert_const!(0 <= rnum);

		if rnum <= u32::MAX as i64 {
			if let Some(exponentiation) = self.get().checked_pow(rnum as u32) {
				if let Some(number) = Self::new(exponentiation) {
					return Ok(number);
				}
			}
		}

		Err(MathError::Overflow { func: '^', lhs: self, rhs })
	}
}
