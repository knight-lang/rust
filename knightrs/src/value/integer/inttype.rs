use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler};
use rand::prelude::*;
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

/// The backing type for [`Integer`](super::Integer)s in Knight.
///
/// Strictly speaking, Knight only requires 32 bit integers, and doesn't specify overflow. This
/// implementation goes beyond that requirement and also allows you to use 64 bit integers.
///
/// For all the functions, `None` should be returned if integer over/underflow happened and should
/// be reported.
///
/// The implementations on [`i32`] and [`i64`] are wrapping operations. To use checked math, use the
/// [`Checked`] type.
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
{
	/// The value `0` for this type.
	const ZERO: Self;

	/// The value `1` for this type.
	const ONE: Self;

	/// The maximum possible value for this type.
	const MAX: Self;

	/// The minimum possible value for this type.
	const MIN: Self;

	/// Gets the log10 of `self`.
	fn log10(self) -> usize;

	/// Negates `self`.
	///
	/// If the implementation checks for under/overflows and one occurs, `None` should be returned.
	fn negate(self) -> Option<Self>;

	/// Adds `self` to `augend`.
	///
	/// If the implementation checks for under/overflows and one occurs, `None` should be returned.
	fn add(self, augend: Self) -> Option<Self>;

	/// Subtracts `minuend` from `self`.
	///
	/// If the implementation checks for under/overflows and one occurs, `None` should be returned.
	fn subtract(self, minuend: Self) -> Option<Self>;

	/// Multiplies `self` by `multiplier`.
	///
	/// If the implementation checks for under/overflows and one occurs, `None` should be returned.
	fn multiply(self, multiplier: Self) -> Option<Self>;

	/// Divides `self` by `divisor`.
	///
	/// The case when `divisor` is zero is handled already by [`Integer::divide`].
	///
	/// If the implementation checks for under/overflows and one occurs, `None` should be returned.
	fn divide(self, divisor: Self) -> Option<Self>;

	/// Gets the remainder of `self` divided by `base`.
	///
	/// The case when `base` is zero is handled already by [`Integer::remainder`].
	///
	/// If the implementation checks for under/overflows and one occurs, `None` should be returned.
	fn remainder(self, base: Self) -> Option<Self>;

	/// Exponentiates `self` by `power`.
	///
	/// If the implementation checks for under/overflows and one occurs, `None` should be returned.
	fn power(self, power: u32) -> Option<Self>;
}

macro_rules! impl_wrapping_int {
	($($ty:ty),*) => {$(
	#[doc = concat!("Implements [`IntType`] for [`", stringify!($ty), "`] by doing wrapping arithmetic")]
	impl IntType for $ty {
		const ZERO: Self = 0;
		const ONE: Self = 1;
		const MAX: Self = Self::MAX;
		const MIN: Self = Self::MIN;

		#[inline(always)]
		fn log10(self) -> usize {
			self.ilog10() as usize
		}

		#[inline(always)]
		fn negate(self) -> Option<Self> {
			Some(self.wrapping_neg())
		}

		#[inline(always)]
		fn add(self, rhs: Self) -> Option<Self> {
			Some(self.wrapping_add(rhs))
		}

		#[inline(always)]
		fn subtract(self, rhs: Self) -> Option<Self> {
			Some(self.wrapping_sub(rhs))
		}

		#[inline(always)]
		fn multiply(self, rhs: Self) -> Option<Self> {
			Some(self.wrapping_mul(rhs))
		}

		#[inline(always)]
		fn divide(self, rhs: Self) -> Option<Self> {
			Some(self.wrapping_div(rhs))
		}

		#[inline(always)]
		fn remainder(self, rhs: Self) -> Option<Self> {
			Some(self.wrapping_rem(rhs))
		}

		#[inline(always)]
		fn power(self, rhs: u32) -> Option<Self> {
			Some(self.wrapping_pow(rhs))
		}
	})*};
}

impl_wrapping_int!(i32, i64);

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Checked<I>(I);

impl<I: Debug> Debug for Checked<I> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl<I: Display> Display for Checked<I> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl<I: From<i32>> From<i32> for Checked<I> {
	fn from(inp: i32) -> Self {
		Self(I::from(inp))
	}
}

impl<I: Into<i64>> From<Checked<I>> for i64 {
	fn from(inp: Checked<I>) -> Self {
		inp.0.into()
	}
}

impl<I: FromStr> FromStr for Checked<I> {
	type Err = I::Err;

	fn from_str(src: &str) -> std::result::Result<Self, Self::Err> {
		src.parse().map(Self)
	}
}

impl<T: TryInto<i32>> TryFrom<Checked<T>> for i32 {
	type Error = T::Error;

	fn try_from(inp: Checked<T>) -> std::result::Result<Self, Self::Error> {
		inp.0.try_into()
	}
}

impl<T: TryFrom<i64>> TryFrom<i64> for Checked<T> {
	type Error = T::Error;

	fn try_from(inp: i64) -> std::result::Result<Self, Self::Error> {
		T::try_from(inp).map(Self)
	}
}

impl<T: TryFrom<usize>> TryFrom<usize> for Checked<T> {
	type Error = T::Error;

	fn try_from(inp: usize) -> std::result::Result<Self, Self::Error> {
		T::try_from(inp).map(Self)
	}
}

#[derive(Clone, Copy, Debug)]
pub struct UniformCheckedIntType<T>(UniformInt<T>);

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
			fn negate(self) -> Option<Self> {
				self.0.checked_neg().map(Self)
			}

			#[inline]
			fn add(self, rhs: Self) -> Option<Self> {
				self.0.checked_add(rhs.0).map(Self)
			}

			#[inline]
			fn subtract(self, rhs: Self) -> Option<Self> {
				self.0.checked_sub(rhs.0).map(Self)
			}

			#[inline]
			fn multiply(self, rhs: Self) -> Option<Self> {
				self.0.checked_mul(rhs.0).map(Self)
			}

			#[inline]
			fn divide(self, rhs: Self) -> Option<Self> {
				self.0.checked_div(rhs.0).map(Self)
			}

			#[inline]
			fn remainder(self, rhs: Self) -> Option<Self> {
				self.0.checked_rem(rhs.0).map(Self)
			}

			#[inline]
			fn power(self, rhs: u32) -> Option<Self> {
				self.0.checked_pow(rhs).map(Self)
			}
		}
	)*};
}

impl_checked_int_type!(i32, i64);
