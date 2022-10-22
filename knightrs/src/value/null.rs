use crate::parse::{self, Parsable, Parser};
use crate::value::{Boolean, Integer, List, NamedType, Text, ToBoolean, ToInteger, ToList, ToText};
use crate::{Environment, Result};
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

impl<I> Parsable<I> for Null {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I>) -> parse::Result<Option<Self>> {
		if parser.advance_if('N').is_none() {
			return Ok(None);
		}

		parser.strip_keyword_function();

		Ok(Some(Self))
	}
}

impl<I> ToBoolean<I> for Null {
	/// Simply returns `false`.
	fn to_boolean(&self, _: &mut Environment<I>) -> Result<Boolean> {
		Ok(Boolean::default())
	}
}

impl<I: Default> ToInteger<I> for Null {
	/// Simply returns zero.
	fn to_integer(&self, _: &mut Environment<I>) -> Result<Integer<I>> {
		Ok(Integer::default())
	}
}

impl<I> ToList<I> for Null {
	/// Simply returns an empty [`List`].
	fn to_list(&self, _: &mut Environment<I>) -> Result<List<I>> {
		Ok(List::default())
	}
}

impl<I> ToText<I> for Null {
	/// Simply returns an empty [`Text`].
	fn to_text(&self, _: &mut Environment<I>) -> Result<Text> {
		Ok(Text::default())
	}
}
