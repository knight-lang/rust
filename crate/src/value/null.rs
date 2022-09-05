use crate::value::{
	Boolean, Integer, KnightType, List, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::Result;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null;

impl KnightType<'_> for Null {
	const TYPENAME: &'static str = "Null";
}

impl ToBoolean for Null {
	fn to_boolean(&self) -> Result<Boolean> {
		Ok(false)
	}
}

impl ToInteger for Null {
	fn to_integer(&self) -> Result<Integer> {
		Ok(Integer::ZERO)
	}
}

impl<'e> ToList<'e> for Null {
	fn to_list(&self) -> Result<List<'e>> {
		Ok(List::EMPTY)
	}
}

impl ToText for Null {
	fn to_text(&self) -> Result<Text> {
		Ok(Text::default())
	}
}