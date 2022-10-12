use crate::parse::{self, Parsable, Parser};
use crate::value::integer::IntType;
use crate::value::text::{Encoding, TextSlice};
use crate::value::{Integer, List, NamedType, Text, ToInteger, ToList, ToText};
use crate::{Environment, Result};

/// The boolean type within Knight.
pub type Boolean = bool;

/// Represents the ability to be converted to a [`Boolean`].
pub trait ToBoolean<I, E> {
	/// Converts `self` to a [`Boolean`].
	fn to_boolean(&self, env: &mut Environment<I, E>) -> Result<Boolean>;
}

impl NamedType for Boolean {
	const TYPENAME: &'static str = "Boolean";
}

impl<I, E: Encoding> Parsable<I, E> for Boolean {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> parse::Result<Option<Self>> {
		let Some(which) = parser.advance_if(|chr| chr == 'T' || chr == 'F') else {
			return Ok(None);
		};

		parser.strip_keyword_function();

		Ok(Some(which == 'T'))
	}
}

impl<I, E> ToBoolean<I, E> for Boolean {
	/// Simply returns `self`.
	#[inline]
	fn to_boolean(&self, _: &mut Environment<I, E>) -> Result<Self> {
		Ok(*self)
	}
}

impl<I: IntType, E> ToInteger<I, E> for Boolean {
	/// Returns `1` for true and `0` for false.
	#[inline]
	fn to_integer(&self, _: &mut Environment<I, E>) -> Result<Integer<I>> {
		if *self {
			Ok(Integer::ONE)
		} else {
			Ok(Integer::ZERO)
		}
	}
}

impl<I, E> ToList<I, E> for Boolean {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	fn to_list(&self, _: &mut Environment<I, E>) -> Result<List<I, E>> {
		if *self {
			Ok(List::boxed((*self).into()))
		} else {
			Ok(List::EMPTY)
		}
	}
}

impl<I, E> ToText<I, E> for Boolean {
	/// Returns `"true"` for true and `"false"` for false.
	fn to_text(&self, _: &mut Environment<I, E>) -> Result<Text<E>> {
		// const TRUE_TEXT: &TextSlice<E> = unsafe { TextSlice::new_unchecked("true") };
		// const FALSE_TEXT: &TextSlice<E> = unsafe { TextSlice::new_unchecked("false") };

		if *self {
			Ok(unsafe { TextSlice::new_unchecked("true") }.into())
		} else {
			Ok(unsafe { TextSlice::new_unchecked("false") }.into())
		}
	}
}
