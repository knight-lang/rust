use crate::{Environment, Value, Number, Boolean, Text};

pub use try_traits::ops::{TryAdd, TrySub, TryMul, TryDiv, TryRem, TryNeg};
pub use try_traits::cmp::{TryPartialEq, TryEq, TryPartialOrd, TryOrd};
pub use std::convert::Infallible;

pub trait TryPow<Rhs=Self> {
	type Error;
	type Output;

	fn try_pow(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

//// A trait that represents the ability to run something.
pub trait Runnable<'env> {
	/// Executes `self`.
	fn run(&self, env: &'env Environment) -> crate::Result<Value<'env>>;
}

/// A trait that represents the fact that nothing happens when a type is [run](Runnable::run).
pub trait Idempotent<'env> : Clone + Into<Value<'env>> {}

impl<'env, T: Idempotent<'env>> Runnable<'env> for T {
	/// Simply clones `self` and converts it to a [`Value`].
	#[inline]
	fn run(&self, _env: &'env Environment) -> crate::Result<Value<'env>> {
		Ok(self.clone().into())
	}
}

pub trait ToBoolean {
	type Error: Into<crate::Error>;

	fn to_boolean(&self) -> Result<Boolean, Self::Error>;
}

pub trait ToText<'a> {
	type Error: Into<crate::Error>;
	type Output: std::borrow::Borrow<Text>;

	fn to_text(&'a self) -> Result<Self::Output, Self::Error>;
}

pub trait ToNumber {
	type Error: Into<crate::Error>;

	fn to_number(&self) -> Result<Number, Self::Error>;
}

impl<T: Into<Boolean> + Clone> ToBoolean for T {
	type Error = Infallible;

	fn to_boolean(&self) -> Result<Boolean, Self::Error> {
		Ok(self.clone().into())
	}
}

impl<T: Into<Number> + Clone> ToNumber for T {
	type Error = Infallible;

	fn to_number(&self) -> Result<Number, Self::Error> {
		Ok(self.clone().into())
	}
}