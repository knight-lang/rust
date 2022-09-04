use crate::value::{Boolean, List, ToBoolean, ToList};
use crate::{Error, Result};
use std::fmt::{self, Display, Formatter};

pub trait ToInteger {
	fn to_integer(&self) -> Result<Integer>;
}

/// The number type within Knight.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer(Inner);

cfg_if! {
	if #[cfg(feature = "strict-numbers")] {
		type Inner = i32;
	} else {
		type Inner = i64;
	}
}

impl Display for Integer {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl Integer {
	pub const ZERO: Self = Self(0);
	pub const ONE: Self = Self(0);

	pub fn inner(self) -> Inner {
		self.0
	}

	pub const fn is_zero(self) -> bool {
		self.0 == 0
	}

	pub const fn is_negative(self) -> bool {
		self.0.is_negative()
	}

	pub fn negate(self) -> Result<Self> {
		if cfg!(feature = "strict-numbers") {
			self.0.checked_neg().map(Self).ok_or(Error::IntegerOverflow)
		} else {
			Ok(Self(self.0.wrapping_neg()))
		}
	}

	fn binary_op(
		self,
		rhs: Self,
		checked: impl FnOnce(Inner, Inner) -> Option<Inner>,
		wrapping: impl FnOnce(Inner, Inner) -> Inner,
	) -> Result<Self> {
		if cfg!(feature = "strict-numbers") {
			(checked)(self.0, rhs.0).map(Self).ok_or(Error::IntegerOverflow)
		} else {
			Ok(Self((wrapping)(self.0, rhs.0)))
		}
	}

	pub fn add(self, rhs: Self) -> Result<Self> {
		self.binary_op(rhs, Inner::checked_add, Inner::wrapping_add)
	}

	pub fn subtract(self, rhs: Self) -> Result<Self> {
		self.binary_op(rhs, Inner::checked_sub, Inner::wrapping_sub)
	}

	pub fn multiply(self, rhs: Self) -> Result<Self> {
		self.binary_op(rhs, Inner::checked_mul, Inner::wrapping_mul)
	}

	pub fn divide(self, rhs: Self) -> Result<Self> {
		if rhs.is_zero() {
			return Err(Error::DivisionByZero);
		}

		self.binary_op(rhs, Inner::checked_div, Inner::wrapping_div)
	}

	pub fn modulo(self, rhs: Self) -> Result<Self> {
		if rhs.is_zero() {
			return Err(Error::DivisionByZero);
		}

		if cfg!(feature = "strict-compliance") && rhs.is_negative() {
			return Err(Error::DomainError("modulo by a negative base"));
		}

		self.binary_op(rhs, Inner::checked_rem, Inner::wrapping_rem)
	}

	pub fn power(self, rhs: Self) -> Result<Self> {
		/*
		match (base, rhs.run(env)?.to_integer()?) {
			(_, Integer::MIN..=-1) => return Err(Error::DomainError("negative exponent")),
			(_, 0) => 1.into(),
			(0 | 1, _) => base.into(),

			#[cfg(feature = "checked-overflow")]
			(_, exponent) => {
				let exp =
					exponent.try_conv::<u32>().or(Err(Error::DomainError("negative exponent")))?;
				base.checked_pow(exp).ok_or(Error::IntegerOverflow)?.conv::<Value>()
			}
			#[cfg(not(feature = "checked-overflow"))]
			(_, exponent) => base.wrapping_pow(exponent as u32).conv::<Value>(),
		},*/
		let _ = rhs;
		todo!()
		// if rhs.is_zero() {
		// 	return Err(Error::DivisionByZero);
		// }

		// if cfg!(feature = "strict-compliance") && rhs.is_negative() {
		// 	return Err(Error::DomainError("modulo by a negative base"));
		// }

		// self.binary_op(rhs, Inner::checked_div, Inner::wrapping_div)
	}
}

impl ToInteger for Integer {
	fn to_integer(&self) -> Result<Self> {
		Ok(*self)
	}
}

impl ToBoolean for Integer {
	fn to_boolean(&self) -> Result<Boolean> {
		Ok(!self.is_zero())
	}
}

impl ToList for Integer {
	fn to_list(&self) -> Result<List> {
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

		Ok(digits.into())
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
