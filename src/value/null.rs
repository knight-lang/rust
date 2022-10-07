use crate::value::{Boolean, Integer, List, NamedType, Text, ToBoolean, ToInteger, ToList, ToText};
use crate::Result;
use std::fmt::{self, Debug, Formatter};

/// Represents the `NULL` value within Knight.
///
/// Note that this explicitly doesn't implement [`PartialOrd`]/[`Ord`], as you cant compare `NULL`
/// in knight.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null;

impl Debug for Null {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str("null")
	}
}

impl NamedType for Null {
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
