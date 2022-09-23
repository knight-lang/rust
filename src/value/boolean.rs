use crate::env::Options;
use crate::value::{Integer, List, NamedType, Text, ToInteger, ToList, ToText};
use crate::Result;

/// The boolean type within Knight.
pub type Boolean = bool;

/// Represents the ability to be converted to a [`Boolean`].
pub trait ToBoolean {
	/// Converts `self` to a [`Boolean`].
	fn to_boolean(&self, opts: &Options) -> Result<Boolean>;
}

impl NamedType for Boolean {
	const TYPENAME: &'static str = "Boolean";
}

impl ToBoolean for Boolean {
	/// Simply returns `self`.
	#[inline]
	fn to_boolean(&self, _: &Options) -> Result<Self> {
		Ok(*self)
	}
}

impl ToInteger for Boolean {
	/// Returns `1` for true and `0` for false.
	#[inline]
	fn to_integer(&self, _: &Options) -> Result<Integer> {
		Ok((*self).into())
	}
}

impl<'e> ToList<'e> for Boolean {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	fn to_list(&self, _: &Options) -> Result<List<'e>> {
		if *self {
			Ok(List::boxed((*self).into()))
		} else {
			Ok(List::EMPTY)
		}
	}
}

impl ToText for Boolean {
	/// Returns `"true"` for true and `"false"` for false.
	fn to_text(&self, _: &Options) -> Result<Text> {
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
