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
	/// Simple returns `false`.
	#[inline]
	fn to_boolean(&self) -> Result<Boolean> {
		Ok(Boolean::default())
	}
}

impl ToInteger for Null {
	/// Simple returns zero.
	#[inline]
	fn to_integer(&self) -> Result<Integer> {
		Ok(Integer::default())
	}
}

impl<'e> ToList<'e> for Null {
	/// Simple returns an empty [`List`].
	#[inline]
	fn to_list(&self) -> Result<List<'e>> {
		Ok(List::default())
	}
}

impl ToText for Null {
	/// Simple returns an empty [`Text`].
	#[inline]
	fn to_text(&self) -> Result<Text> {
		Ok(Text::default())
	}
}
