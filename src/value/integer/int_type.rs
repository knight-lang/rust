use crate::{Error, Result};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;

pub trait IntType:
	Sized
	+ 'static
	+ Debug
	+ Clone
	+ Copy
	+ PartialEq
	+ Eq
	+ Ord
	+ Hash
	+ Default
	+ Display
	+ From<i32>
	+ Into<i32>
	+ TryFrom<usize, Error = Error>
	+ TryInto<usize, Error = Error>
{
	const ZERO: Self;
	const ONE: Self;

	fn is_in_bounds(num: i64) -> bool;
	fn is_negative(self) -> bool {
		self < Self::ZERO
	}
	fn as_u32(self) -> Option<u32> {
		<Self as Into<i32>>::into(self).try_into().ok()
	}

	fn negate(self) -> Result<Self>;
	fn add(self, rhs: Self) -> Result<Self>;
	fn subtract(self, rhs: Self) -> Result<Self>;
	fn multiply(self, rhs: Self) -> Result<Self>;
	fn divide(self, rhs: Self) -> Result<Self>;
	fn remainder(self, rhs: Self) -> Result<Self>;
	fn power(self, rhs: u32) -> Result<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Wrapping<T>(T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Checked<T>(T);

impl<T: Display> Display for Wrapping<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl<T: Display> Display for Checked<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl From<i32> for Wrapping<i32> {
	fn from(int: i32) -> Self {
		Self(int)
	}
}
impl From<i32> for Wrapping<i64> {
	fn from(int: i32) -> Self {
		Self(int as i64)
	}
}
impl From<i32> for Checked<i32> {
	fn from(int: i32) -> Self {
		Self(int)
	}
}
impl From<i32> for Checked<i64> {
	fn from(int: i32) -> Self {
		Self(int as i64)
	}
}

impl From<Wrapping<i32>> for i32 {
	fn from(int: Wrapping<i32>) -> Self {
		int.0
	}
}
impl From<Wrapping<i64>> for i32 {
	fn from(int: Wrapping<i64>) -> Self {
		int.0 as i32 // truncate
	}
}
impl From<Checked<i32>> for i32 {
	fn from(int: Checked<i32>) -> Self {
		int.0
	}
}
impl From<Checked<i64>> for i32 {
	fn from(int: Checked<i64>) -> Self {
		int.0 as i32 // truncate
	}
}

impl TryFrom<usize> for Wrapping<i32> {
	type Error = Error;

	fn try_from(inp: usize) -> Result<Self> {
		inp.try_into().map(Self).or(Err(Error::IntegerOverflow))
	}
}
impl TryFrom<usize> for Wrapping<i64> {
	type Error = Error;

	fn try_from(inp: usize) -> Result<Self> {
		inp.try_into().map(Self).or(Err(Error::IntegerOverflow))
	}
}
impl TryFrom<usize> for Checked<i32> {
	type Error = Error;

	fn try_from(inp: usize) -> Result<Self> {
		inp.try_into().map(Self).or(Err(Error::IntegerOverflow))
	}
}
impl TryFrom<usize> for Checked<i64> {
	type Error = Error;

	fn try_from(inp: usize) -> Result<Self> {
		inp.try_into().map(Self).or(Err(Error::IntegerOverflow))
	}
}

impl TryFrom<Wrapping<i32>> for usize {
	type Error = Error;

	fn try_from(inp: Wrapping<i32>) -> Result<Self> {
		inp.0.try_into().or(Err(Error::IntegerOverflow))
	}
}
impl TryFrom<Wrapping<i64>> for usize {
	type Error = Error;

	fn try_from(inp: Wrapping<i64>) -> Result<Self> {
		inp.0.try_into().or(Err(Error::IntegerOverflow))
	}
}
impl TryFrom<Checked<i32>> for usize {
	type Error = Error;

	fn try_from(inp: Checked<i32>) -> Result<Self> {
		inp.0.try_into().or(Err(Error::IntegerOverflow))
	}
}
impl TryFrom<Checked<i64>> for usize {
	type Error = Error;

	fn try_from(inp: Checked<i64>) -> Result<Self> {
		inp.0.try_into().or(Err(Error::IntegerOverflow))
	}
}

impl IntType for Wrapping<i32> {
	const ZERO: Self = Self(0);
	const ONE: Self = Self(1);

	fn is_in_bounds(int: i64) -> bool {
		i32::try_from(int).is_ok()
	}

	fn negate(self) -> Result<Self> {
		Ok(Self(self.0.wrapping_neg()))
	}
	fn add(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_add(rhs.0)))
	}
	fn subtract(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_sub(rhs.0)))
	}
	fn multiply(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_mul(rhs.0)))
	}
	fn divide(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_div(rhs.0)))
	}
	fn remainder(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_rem(rhs.0)))
	}
	fn power(self, rhs: u32) -> Result<Self> {
		Ok(Self(self.0.wrapping_pow(rhs)))
	}
}

impl IntType for Wrapping<i64> {
	const ZERO: Self = Self(0);
	const ONE: Self = Self(1);

	fn is_in_bounds(_int: i64) -> bool {
		true
	}

	fn negate(self) -> Result<Self> {
		Ok(Self(self.0.wrapping_neg()))
	}
	fn add(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_add(rhs.0)))
	}
	fn subtract(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_sub(rhs.0)))
	}
	fn multiply(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_mul(rhs.0)))
	}
	fn divide(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_div(rhs.0)))
	}
	fn remainder(self, rhs: Self) -> Result<Self> {
		Ok(Self(self.0.wrapping_rem(rhs.0)))
	}
	fn power(self, rhs: u32) -> Result<Self> {
		Ok(Self(self.0.wrapping_pow(rhs)))
	}
}

impl IntType for Checked<i32> {
	const ZERO: Self = Self(0);
	const ONE: Self = Self(1);

	fn is_in_bounds(int: i64) -> bool {
		i32::try_from(int).is_ok()
	}

	fn negate(self) -> Result<Self> {
		self.0.checked_neg().map(Self).ok_or(Error::IntegerOverflow)
	}
	fn add(self, rhs: Self) -> Result<Self> {
		self.0.checked_add(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn subtract(self, rhs: Self) -> Result<Self> {
		self.0.checked_sub(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn multiply(self, rhs: Self) -> Result<Self> {
		self.0.checked_mul(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn divide(self, rhs: Self) -> Result<Self> {
		self.0.checked_div(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn remainder(self, rhs: Self) -> Result<Self> {
		self.0.checked_rem(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn power(self, rhs: u32) -> Result<Self> {
		self.0.checked_pow(rhs).map(Self).ok_or(Error::IntegerOverflow)
	}
}

impl IntType for Checked<i64> {
	const ZERO: Self = Self(0);
	const ONE: Self = Self(1);

	fn is_in_bounds(_int: i64) -> bool {
		true
	}

	fn negate(self) -> Result<Self> {
		self.0.checked_neg().map(Self).ok_or(Error::IntegerOverflow)
	}
	fn add(self, rhs: Self) -> Result<Self> {
		self.0.checked_add(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn subtract(self, rhs: Self) -> Result<Self> {
		self.0.checked_sub(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn multiply(self, rhs: Self) -> Result<Self> {
		self.0.checked_mul(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn divide(self, rhs: Self) -> Result<Self> {
		self.0.checked_div(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn remainder(self, rhs: Self) -> Result<Self> {
		self.0.checked_rem(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
	}
	fn power(self, rhs: u32) -> Result<Self> {
		self.0.checked_pow(rhs).map(Self).ok_or(Error::IntegerOverflow)
	}
}
