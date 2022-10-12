use crate::parse::{self, Parsable, Parser};
use crate::value::text::Encoding;
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

impl<I, E: Encoding> Parsable<'_, I, E> for Null {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> parse::Result<Option<Self>> {
		if parser.advance_if('N').is_none() {
			return Ok(None);
		}

		parser.strip_keyword_function();

		Ok(Some(Self))
	}
}

impl<'e, I, E> ToBoolean<'e, I, E> for Null {
	/// Simple returns `false`.
	fn to_boolean(&self, _: &mut Environment<'e, I, E>) -> Result<Boolean> {
		Ok(Boolean::default())
	}
}

impl<'e, I: Default, E> ToInteger<'e, I, E> for Null {
	/// Simple returns zero.
	fn to_integer(&self, _: &mut Environment<'e, I, E>) -> Result<Integer<I>> {
		Ok(Integer::default())
	}
}

impl<'e, I, E> ToList<'e, I, E> for Null {
	/// Simple returns an empty [`List`].
	fn to_list(&self, _: &mut Environment<'e, I, E>) -> Result<List<'e, I, E>> {
		Ok(List::default())
	}
}

impl<'e, I, E> ToText<'e, I, E> for Null {
	/// Simple returns an empty [`Text`].
	fn to_text(&self, _: &mut Environment<'e, I, E>) -> Result<Text<E>> {
		Ok(Text::default())
	}
}
