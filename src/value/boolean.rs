use crate::parse::{self, Parsable, Parser};
use crate::value::integer::IntType;
use crate::value::{text::TextSlice, Integer, List, NamedType, Text, ToInteger, ToList, ToText};
use crate::{Environment, Result};

/// The boolean type within Knight.
pub type Boolean = bool;

/// Represents the ability to be converted to a [`Boolean`].
pub trait ToBoolean<'e, I> {
	/// Converts `self` to a [`Boolean`].
	fn to_boolean(&self, env: &mut Environment<'e, I>) -> Result<Boolean>;
}

impl NamedType for Boolean {
	const TYPENAME: &'static str = "Boolean";
}

impl<I: IntType> Parsable<'_, I> for Boolean {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I>) -> parse::Result<Option<Self>> {
		let Some(which) = parser.advance_if(|chr| chr == 'T' || chr == 'F') else {
			return Ok(None);
		};

		parser.strip_keyword_function();

		Ok(Some(which == 'T'))
	}
}

impl<'e, I> ToBoolean<'e, I> for Boolean {
	/// Simply returns `self`.
	#[inline]
	fn to_boolean(&self, _: &mut Environment<'e, I>) -> Result<Self> {
		Ok(*self)
	}
}

impl<'e, I: IntType> ToInteger<'e, I> for Boolean {
	/// Returns `1` for true and `0` for false.
	#[inline]
	fn to_integer(&self, _: &mut Environment<'e, I>) -> Result<Integer<I>> {
		if *self {
			Ok(Integer::ONE)
		} else {
			Ok(Integer::ZERO)
		}
	}
}

impl<'e, I> ToList<'e, I> for Boolean {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	fn to_list(&self, _: &mut Environment<'e, I>) -> Result<List<'e, I>> {
		if *self {
			Ok(List::boxed((*self).into()))
		} else {
			Ok(List::EMPTY)
		}
	}
}

impl<'e, I> ToText<'e, I> for Boolean {
	/// Returns `"true"` for true and `"false"` for false.
	fn to_text(&self, _: &mut Environment<'e, I>) -> Result<Text> {
		static TRUE_TEXT: &TextSlice = unsafe { TextSlice::new_unchecked("true") };
		static FALSE_TEXT: &TextSlice = unsafe { TextSlice::new_unchecked("false") };

		if *self {
			Ok(TRUE_TEXT.into())
		} else {
			Ok(FALSE_TEXT.into())
		}
	}
}
