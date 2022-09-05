use crate::value::{Integer, KnightType, List, Text, ToInteger, ToList, ToText};
use crate::Result;

/// The boolean type within Knight.
pub type Boolean = bool;

/// Represents the ability to be converted to a [`Boolean`].
pub trait ToBoolean {
	/// Converts `self` to a [`Boolean`].
	fn to_boolean(&self) -> Result<Boolean>;
}

impl KnightType for Boolean {
	const TYPENAME: &'static str = "Boolean";
}

impl ToBoolean for Boolean {
	#[inline]
	fn to_boolean(&self) -> Result<Self> {
		Ok(*self)
	}
}

impl ToInteger for Boolean {
	#[inline]
	fn to_integer(&self) -> Result<Integer> {
		Ok((*self).into())
	}
}

impl ToList for Boolean {
	fn to_list(&self) -> Result<List> {
		if *self {
			Ok(List::boxed((*self).into()))
		} else {
			Ok(List::EMPTY)
		}
	}
}

impl ToText for Boolean {
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
