use crate::env::Environment;
use crate::parse::{self, Parsable, Parser};
use crate::value::{Boolean, Integer, List, NamedType, Text, ToBoolean, ToInteger, ToList, ToText};
use std::fmt::{self, Debug, Formatter};

/// Represents the `NULL` value within Knight.
///
/// Note that this explicitly doesn't implement [`PartialOrd`]/[`Ord`], as you cant compare `NULL`
/// in knight.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null;

impl Debug for Null {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str("null")
	}
}

impl NamedType for Null {
	const TYPENAME: &'static str = "Null";
}

impl Parsable for Null {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
		if parser.advance_if('N').is_none() {
			return Ok(None);
		}

		parser.strip_keyword_function();

		Ok(Some(Self))
	}
}

impl ToBoolean for Null {
	/// Simply returns `false`.
	#[inline]
	fn to_boolean(&self, _: &mut Environment) -> crate::Result<Boolean> {
		Ok(Boolean::default())
	}
}

impl ToInteger for Null {
	/// Simply returns zero.
	#[inline]
	fn to_integer(&self, _: &mut Environment) -> crate::Result<Integer> {
		Ok(Integer::default())
	}
}

impl ToList for Null {
	/// Simply returns an empty [`List`].
	#[inline]
	fn to_list(&self, _: &mut Environment) -> crate::Result<List> {
		Ok(List::default())
	}
}

impl ToText for Null {
	/// Simply returns an empty [`Text`].
	#[inline]
	fn to_text(&self, _: &mut Environment) -> crate::Result<Text> {
		Ok(Text::default())
	}
}
