use crate::text::{ToText, TextRef, TextCow};
use crate::number::{ToNumber, Number};

/// The boolean type within Knight.
pub type Boolean = bool;

pub trait ToBoolean {
	fn to_boolean(&self) -> crate::Result<Boolean>;
}

impl ToBoolean for Boolean {
	fn to_boolean(&self) -> crate::Result<Boolean> {
		Ok(*self)
	}
}

impl ToNumber for Boolean {
	fn to_number(&self) -> crate::Result<Number> {
		const ONE: Number = unsafe { Number::new_unchecked(1) };
		const ZERO: Number = unsafe { Number::new_unchecked(0) };

		Ok(if *self { ONE } else { ZERO })
	}
}