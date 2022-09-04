use crate::value::{Integer, KnightType, List, Text, ToInteger, ToList, ToText};
use crate::Result;

pub type Boolean = bool;

pub trait ToBoolean {
	fn to_boolean(&self) -> Result<Boolean>;
}

impl KnightType for Boolean {
	const TYPENAME: &'static str = "Boolean";
}

impl ToBoolean for Boolean {
	fn to_boolean(&self) -> Result<Self> {
		Ok(*self)
	}
}

impl ToInteger for Boolean {
	fn to_integer(&self) -> Result<Integer> {
		if *self {
			Ok(Integer::ONE)
		} else {
			Ok(Integer::ZERO)
		}
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
