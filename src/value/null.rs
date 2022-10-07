use crate::env::Options;
use crate::value::{Boolean, Integer, List, NamedType, Text, ToBoolean, ToInteger, ToList, ToText};
use crate::Encoding;
use crate::IntType;
use crate::Result;

/// Represents the `NULL` value within Knight.
///
/// Note that this explicitly doesn't implement [`PartialOrd`]/[`Ord`], as you cant compare `NULL`
/// in knight.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null;

impl NamedType for Null {
	const TYPENAME: &'static str = "Null";
}

impl ToBoolean for Null {
	/// Simple returns `false`.
	#[inline]
	fn to_boolean(&self, _: &Options) -> Result<Boolean> {
		Ok(Boolean::default())
	}
}

impl<I: IntType> ToInteger<I> for Null {
	/// Simple returns zero.
	#[inline]
	fn to_integer(&self, _: &Options) -> Result<Integer<I>> {
		Ok(Integer::default())
	}
}

impl<'e, E: Encoding, I: IntType> ToList<'e, E, I> for Null {
	/// Simple returns an empty [`List`].
	#[inline]
	fn to_list(&self, _: &Options) -> Result<List<'e, E, I>> {
		Ok(List::default())
	}
}

impl<E: Encoding> ToText<E> for Null {
	/// Simple returns an empty [`Text`].
	#[inline]
	fn to_text(&self, _: &Options) -> Result<Text<E>> {
		Ok(Text::default())
	}
}
