use crate::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

pub trait IntType:
	Default
	+ Copy
	+ Eq
	+ Ord
	+ Debug
	+ Display
	+ std::hash::Hash
	+ From<i32>
	+ Into<i64>
	+ FromStr
	+ TryInto<i32>
	+ TryFrom<i64>
	+ TryFrom<usize>
	+ crate::containers::MaybeSendSync
	+ 'static
{
	const ZERO: Self;
	const ONE: Self;

	fn log10(self) -> usize;
	fn negate(self) -> crate::Result<Self>;
	fn add(self, rhs: Self) -> crate::Result<Self>;
	fn subtract(self, rhs: Self) -> crate::Result<Self>;
	fn multiply(self, rhs: Self) -> crate::Result<Self>;
	fn divide(self, rhs: Self) -> crate::Result<Self>;
	fn remainder(self, rhs: Self) -> crate::Result<Self>;
	fn power(self, rhs: u32) -> crate::Result<Self>;
}

macro_rules! create_int_type {
	($(#[$meta:meta])* $name:ident) => {
		#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct $name<I>(I);

		impl<I: Debug> Debug for $name<I> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				Debug::fmt(&self.0, f)
			}
		}

		impl<I: Display> Display for $name<I> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				Display::fmt(&self.0, f)
			}
		}

		impl<I: From<i32>> From<i32> for $name<I> {
			fn from(inp: i32) -> Self {
				Self(I::from(inp))
			}
		}

		impl<I: Into<i64>> From<$name<I>> for i64 {
			fn from(inp: $name<I>) -> Self {
				inp.0.into()
			}
		}

		impl<I: FromStr> FromStr for $name<I> {
			type Err = I::Err;

			fn from_str(src: &str) -> Result<Self, Self::Err> {
				src.parse().map(Self)
			}
		}

		impl<T: TryInto<i32>> TryFrom<$name<T>> for i32 {
			type Error = T::Error;

			fn try_from(inp: $name<T>) -> Result<Self, Self::Error> {
				inp.0.try_into()
			}
		}

		impl<T: TryFrom<i64>> TryFrom<i64> for $name<T> {
			type Error = T::Error;

			fn try_from(inp: i64) -> Result<Self, Self::Error> {
				T::try_from(inp).map(Self)
			}
		}

		impl<T: TryFrom<usize>> TryFrom<usize> for $name<T> {
			type Error = T::Error;

			fn try_from(inp: usize) -> Result<Self, Self::Error> {
				T::try_from(inp).map(Self)
			}
		}
	};
}

macro_rules! impl_checked_int_type {
	($($ty:ty),*) => {$(
		impl IntType for Checked<$ty> {
			const ZERO: Self = Self(0);
			const ONE: Self = Self(1);

			#[inline]
			fn log10(mut self) -> usize {
				// TODO: integer base10 when that comes out.
				let mut log = 0;
				while self.0 != 0 {
					log += 1;
					self.0 /= 10;
				}

				log
			}

			#[inline]
			fn negate(self) -> crate::Result<Self> {
				self.0.checked_neg().map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn add(self, rhs: Self) -> crate::Result<Self> {
				self.0.checked_add(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn subtract(self, rhs: Self) -> crate::Result<Self> {
				self.0.checked_sub(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn multiply(self, rhs: Self) -> crate::Result<Self> {
				self.0.checked_mul(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn divide(self, rhs: Self) -> crate::Result<Self> {
				self.0.checked_div(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn remainder(self, rhs: Self) -> crate::Result<Self> {
				self.0.checked_rem(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn power(self, rhs: u32) -> crate::Result<Self> {
				self.0.checked_pow(rhs).map(Self).ok_or(Error::IntegerOverflow)
			}
		}
	)*};
}
macro_rules! impl_wrapping_int_type {
	($($ty:ty),*) => {$(
		impl IntType for Wrapping<$ty> {
			const ZERO: Self = Self(0);
			const ONE: Self = Self(1);

			#[inline]
			fn log10(mut self) -> usize {
				// TODO: integer base10 when that comes out.
				let mut log = 0;
				while self.0 != 0 {
					log += 1;
					self.0 /= 10;
				}

				log
			}

			#[inline]
			fn negate(self) -> crate::Result<Self> {
				Ok(Self(self.0.wrapping_neg()))
			}

			#[inline]
			fn add(self, rhs: Self) -> crate::Result<Self> {
				Ok(Self(self.0.wrapping_add(rhs.0)))
			}

			#[inline]
			fn subtract(self, rhs: Self) -> crate::Result<Self> {
				Ok(Self(self.0.wrapping_sub(rhs.0)))
			}

			#[inline]
			fn multiply(self, rhs: Self) -> crate::Result<Self> {
				Ok(Self(self.0.wrapping_mul(rhs.0)))
			}

			#[inline]
			fn divide(self, rhs: Self) -> crate::Result<Self> {
				Ok(Self(self.0.wrapping_div(rhs.0)))
			}

			#[inline]
			fn remainder(self, rhs: Self) -> crate::Result<Self> {
				Ok(Self(self.0.wrapping_rem(rhs.0)))
			}

			#[inline]
			fn power(self, rhs: u32) -> crate::Result<Self> {
				Ok(Self(self.0.wrapping_pow(rhs)))
			}
		}
	)*};
}

create_int_type!(Wrapping);
create_int_type!(Checked);

impl_wrapping_int_type!(i32, i64);
impl_checked_int_type!(i32, i64);
