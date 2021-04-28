use crate::{Value, value::Idempotent, Boolean, Text};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Number(i64);

#[derive(Debug)]
pub struct InvalidI64(i64);

impl Display for InvalidI64 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "the number '{}' is too large to fit in a Number", self.0)	
	}
}

impl std::error::Error for InvalidI64 {}

const fn is_number_valid(num: i64) -> bool {
	(num >> 4) << 4 == num
}

impl Display for Number {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl Number {
	pub const fn new(num: i64) -> Option<Self> {
		if is_number_valid(num) {
			Some(unsafe { Self::new_unchecked(num) })
		} else {
			None
		}
	}

	pub const unsafe fn new_unchecked(num: i64) -> Self {
		debug_assert_const!(is_number_valid(num));

		Self(num)
	}

	pub const fn into_value_const(self) -> Value {
		unsafe {
			Value::from_bytes((self.0 as u64) << 4)
		}
	}
}

impl From<Number> for Value {
	#[inline]
	fn from(num: Number) -> Self {
		num.into_value_const()
	}
}

impl From<Number> for Boolean {
	#[inline]
	fn from(num: Number) -> Self {
		Self::new(num.0 != 0)
	}
}

impl From<Number> for Text {
	#[inline]
	fn from(num: Number) -> Self {
		let _ = num;
		todo!()
		// Self::new(std::borrow::Cow::Owned(num.to_string().into_boxed_str().into_boxed_bytes()))
	}
}

impl Idempotent for Number {}
