use crate::env::Flags;
use crate::{Error, Result};
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler};
use rand::prelude::*;
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
	+ SampleUniform
	+ 'static
{
	const MAX: Self;
	const MIN: Self;
	const ZERO: Self;
	const ONE: Self;

	fn log10(self) -> usize;
	fn negate(self, flags: &Flags) -> Result<Self>;
	fn add(self, rhs: Self, flags: &Flags) -> Result<Self>;
	fn subtract(self, rhs: Self, flags: &Flags) -> Result<Self>;
	fn multiply(self, rhs: Self, flags: &Flags) -> Result<Self>;
	// you can assume `rhs` is nonzero
	fn divide(self, rhs: Self, flags: &Flags) -> Result<Self>;
	// you can assume `rhs` is nonzero
	fn remainder(self, rhs: Self, flags: &Flags) -> Result<Self>;
	fn power(self, rhs: u32, flags: &Flags) -> Result<Self>;
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

			fn from_str(src: &str) -> std::result::Result<Self, Self::Err> {
				src.parse().map(Self)
			}
		}

		impl<T: TryInto<i32>> TryFrom<$name<T>> for i32 {
			type Error = T::Error;

			fn try_from(inp: $name<T>) -> std::result::Result<Self, Self::Error> {
				inp.0.try_into()
			}
		}

		impl<T: TryFrom<i64>> TryFrom<i64> for $name<T> {
			type Error = T::Error;

			fn try_from(inp: i64) -> std::result::Result<Self, Self::Error> {
				T::try_from(inp).map(Self)
			}
		}

		impl<T: TryFrom<usize>> TryFrom<usize> for $name<T> {
			type Error = T::Error;

			fn try_from(inp: usize) -> std::result::Result<Self, Self::Error> {
				T::try_from(inp).map(Self)
			}
		}
	};
}

#[derive(Clone, Copy, Debug)]
pub struct UniformCheckedIntType<T>(UniformInt<T>);

#[derive(Clone, Copy, Debug)]
pub struct UniformWrappingIntType<T>(UniformInt<T>);

macro_rules! impl_checked_int_type {
	($($ty:ty),*) => {$(
		impl SampleUniform for Checked<$ty> {
			type Sampler = UniformCheckedIntType<$ty>;
		}

		impl UniformSampler for UniformCheckedIntType<$ty> {
			type X = Checked<$ty>;
			fn new<B1, B2>(low: B1, high: B2) -> Self
			where
				B1: SampleBorrow<Self::X>,
				B2: SampleBorrow<Self::X>,
			{
				UniformCheckedIntType(UniformInt::<$ty>::new(low.borrow().0, high.borrow().0))
			}
			fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
			where
				B1: SampleBorrow<Self::X>,
				B2: SampleBorrow<Self::X>,
			{
				UniformCheckedIntType(UniformInt::<$ty>::new_inclusive(low.borrow().0, high.borrow().0))
			}
			fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
				Checked(self.0.sample(rng))
			}
		}

		impl IntType for Checked<$ty> {
			const MIN: Self = Self(<$ty>::MIN);
			const MAX: Self = Self(<$ty>::MAX);
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
			fn negate(self, _: &Flags) -> Result<Self> {
				self.0.checked_neg().map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn add(self, rhs: Self, _: &Flags) -> Result<Self> {
				self.0.checked_add(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn subtract(self, rhs: Self, _: &Flags) -> Result<Self> {
				self.0.checked_sub(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn multiply(self, rhs: Self, _: &Flags) -> Result<Self> {
				self.0.checked_mul(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn divide(self, rhs: Self, _: &Flags) -> Result<Self> {
				self.0.checked_div(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn remainder(self, rhs: Self, _: &Flags) -> Result<Self> {
				self.0.checked_rem(rhs.0).map(Self).ok_or(Error::IntegerOverflow)
			}

			#[inline]
			fn power(self, rhs: u32, _: &Flags) -> Result<Self> {
				self.0.checked_pow(rhs).map(Self).ok_or(Error::IntegerOverflow)
			}
		}
	)*};
}
macro_rules! impl_wrapping_int_type {
	($($ty:ty),*) => {$(
		impl SampleUniform for Wrapping<$ty> {
			type Sampler = UniformWrappingIntType<$ty>;
		}

		impl UniformSampler for UniformWrappingIntType<$ty> {
			type X = Wrapping<$ty>;
			fn new<B1, B2>(low: B1, high: B2) -> Self
			where
				B1: SampleBorrow<Self::X>,
				B2: SampleBorrow<Self::X>,
			{
				UniformWrappingIntType(UniformInt::<$ty>::new(low.borrow().0, high.borrow().0))
			}
			fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
			where
				B1: SampleBorrow<Self::X>,
				B2: SampleBorrow<Self::X>,
			{
				UniformWrappingIntType(UniformInt::<$ty>::new_inclusive(low.borrow().0, high.borrow().0))
			}
			fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
				Wrapping(self.0.sample(rng))
			}
		}

		impl IntType for Wrapping<$ty> {
			const MIN: Self = Self(<$ty>::MIN);
			const MAX: Self = Self(<$ty>::MAX);
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
			fn negate(self, _: &Flags) -> Result<Self> {
				Ok(Self(self.0.wrapping_neg()))
			}

			#[inline]
			fn add(self, rhs: Self, _: &Flags) -> Result<Self> {
				Ok(Self(self.0.wrapping_add(rhs.0)))
			}

			#[inline]
			fn subtract(self, rhs: Self, _: &Flags) -> Result<Self> {
				Ok(Self(self.0.wrapping_sub(rhs.0)))
			}

			#[inline]
			fn multiply(self, rhs: Self, _: &Flags) -> Result<Self> {
				Ok(Self(self.0.wrapping_mul(rhs.0)))
			}

			#[inline]
			fn divide(self, rhs: Self, _: &Flags) -> Result<Self> {
				Ok(Self(self.0.wrapping_div(rhs.0)))
			}

			#[inline]
			fn remainder(self, rhs: Self, _: &Flags) -> Result<Self> {
				Ok(Self(self.0.wrapping_rem(rhs.0)))
			}

			#[inline]
			fn power(self, rhs: u32, _: &Flags) -> Result<Self> {
				Ok(Self(self.0.wrapping_pow(rhs)))
			}
		}
	)*};
}

create_int_type!(Wrapping);
create_int_type!(Checked);

impl_wrapping_int_type!(i32, i64);
impl_checked_int_type!(i32, i64);
