use super::Integer;
use crate::{Error, Result};
use std::marker::PhantomData;

pub trait IntType: Sized + 'static {
	fn is_in_bounds(int: i64) -> bool;
	fn negate(int: Integer<Self>) -> Result<Integer<Self>>;
}

pub struct Wrapping<T>(PhantomData<T>);
pub struct Checked<T>(PhantomData<T>);

impl IntType for Wrapping<i32> {
	fn is_in_bounds(int: i64) -> bool {
		i32::try_from(int).is_ok()
	}

	fn negate(int: Integer<Self>) -> Result<Integer<Self>> {
		Ok(Integer(int.0.wrapping_neg(), PhantomData))
	}
}

impl IntType for Wrapping<i64> {
	fn is_in_bounds(_int: i64) -> bool {
		true
	}

	fn negate(int: Integer<Self>) -> Result<Integer<Self>> {
		Ok(Integer(int.0.wrapping_neg(), PhantomData))
	}
}

impl IntType for Checked<i32> {
	fn is_in_bounds(int: i64) -> bool {
		i32::try_from(int).is_ok()
	}

	fn negate(int: Integer<Self>) -> Result<Integer<Self>> {
		int.0.checked_neg().map(|x| Integer(x, PhantomData)).ok_or(Error::IntegerOverflow)
	}
}

impl IntType for Checked<i64> {
	fn is_in_bounds(_int: i64) -> bool {
		true
	}

	fn negate(int: Integer<Self>) -> Result<Integer<Self>> {
		int.0.checked_neg().map(|x| Integer(x, PhantomData)).ok_or(Error::IntegerOverflow)
	}
}
