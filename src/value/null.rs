use crate::parse::{self, Parsable, Parser};
use crate::value::integer::IntType;
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

impl<I: IntType> Parsable<'_, I> for Null {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I>) -> parse::Result<Option<Self>> {
		if parser.advance_if('N').is_none() {
			return Ok(None);
		}

		parser.strip_keyword_function();

		Ok(Some(Self))
	}
}

impl<'e, I: IntType> ToBoolean<'e, I> for Null {
	/// Simple returns `false`.
	#[inline]
	fn to_boolean(&self, _: &mut Environment<'e, I>) -> Result<Boolean> {
		Ok(Boolean::default())
	}
}

impl<'e, I: IntType> ToInteger<'e, I> for Null {
	/// Simple returns zero.
	#[inline]
	fn to_integer(&self, _: &mut Environment<'e, I>) -> Result<Integer<I>> {
		Ok(Integer::default())
	}
}

impl<'e, I: IntType> ToList<'e, I> for Null {
	/// Simple returns an empty [`List`].
	#[inline]
	fn to_list(&self, _: &mut Environment<'e, I>) -> Result<List<'e, I>> {
		Ok(List::default())
	}
}

impl<'e, I: IntType> ToText<'e, I> for Null {
	/// Simple returns an empty [`Text`].
	#[inline]
	fn to_text(&self, _: &mut Environment<'e, I>) -> Result<Text> {
		Ok(Text::default())
	}
}
