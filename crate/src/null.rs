use crate::text::{ToText, Text, TextCow};
use crate::number::{ToNumber, Number};
use crate::boolean::{ToBoolean, Boolean};

// notably not `ParitalOrd`/`ORd`, as Knight says null isnt comparable
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Null;

impl ToText for Null {
	fn to_text(&self) -> crate::Result<TextCow<'static>> {
		todo!()
	}
}

impl ToBoolean for Null {
	fn to_boolean(&self) -> crate::Result<Boolean> {
		Ok(false)
	}
}

impl ToNumber for Null {
	fn to_number(&self) -> crate::Result<Number> {
		Ok(Number::ZERO)
	}
}
