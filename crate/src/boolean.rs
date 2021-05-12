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
		Ok(if *self { Number::ONE } else { Number::ZERO })
	}
}

impl ToText for Boolean {
	fn to_text(&self) -> crate::Result<TextCow<'static>>  {
		todo!()
	}
}