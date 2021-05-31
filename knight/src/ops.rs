pub use try_traits::ops::{TryAdd, TrySub, TryMul, TryDiv, TryRem};
pub use try_traits::cmp::{TryNeg, TryPartialEq, TryOrd};

pub trait TryPow<Rhs=Self> {
	type Error;
	type Output;
	fn try_pow(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}
