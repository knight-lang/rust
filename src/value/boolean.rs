use crate::parse::{self, Parsable, Parser};
use crate::value::{Integer, List, NamedType, Text, ToInteger, ToList, ToText};
use crate::Result;

/// The boolean type within Knight.
pub type Boolean = bool;

/// Represents the ability to be converted to a [`Boolean`].
pub trait ToBoolean {
	/// Converts `self` to a [`Boolean`].
	fn to_boolean(&self) -> Result<Boolean>;
}

impl NamedType for Boolean {
	const TYPENAME: &'static str = "Boolean";
}

impl Parsable<'_, '_> for Boolean {
	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
		let Some(which) = parser.advance_if(|chr| chr == 'T' || chr == 'F') else {
			return Ok(None);
		};

		parser.strip_keyword_function();

		Ok(Some(which == 'T'))
	}
}

impl ToBoolean for Boolean {
	/// Simply returns `self`.
	#[inline]
	fn to_boolean(&self) -> Result<Self> {
		Ok(*self)
	}
}

impl ToInteger for Boolean {
	/// Returns `1` for true and `0` for false.
	#[inline]
	fn to_integer(&self) -> Result<Integer> {
		Ok((*self).into())
	}
}

impl<'e> ToList<'e> for Boolean {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	fn to_list(&self) -> Result<List<'e>> {
		if *self {
			Ok(List::boxed((*self).into()))
		} else {
			Ok(List::EMPTY)
		}
	}
}

impl ToText for Boolean {
	/// Returns `"true"` for true and `"false"` for false.
	fn to_text(&self) -> Result<Text> {
		use crate::text::TextSlice;

		const TRUE_TEXT: &TextSlice = unsafe { TextSlice::new_unchecked("true") };
		const FALSE_TEXT: &TextSlice = unsafe { TextSlice::new_unchecked("false") };

		if *self {
			Ok(TRUE_TEXT.into())
		} else {
			Ok(FALSE_TEXT.into())
		}
	}
}
