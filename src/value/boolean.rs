use crate::env::Options;
use crate::value::{Integer, List, NamedType, Text, ToInteger, ToList, ToText};
use crate::Encoding;
use crate::IntType;
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

impl<I: IntType> ToInteger<I> for Boolean {
	/// Returns `1` for true and `0` for false.
	#[inline]
	fn to_integer(&self, _: &Options) -> Result<Integer<I>> {
		if *self {
			Ok(Integer::ONE)
		} else {
			Ok(Integer::ZERO)
		}
	}
}

impl<'e, E: Encoding, I: IntType> ToList<'e, E, I> for Boolean {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	fn to_list(&self, _: &Options) -> Result<List<'e, E, I>> {
		if *self {
			Ok(List::boxed((*self).into()))
		} else {
			Ok(List::EMPTY)
		}
	}
}

impl<E: Encoding> ToText<E> for Boolean {
	/// Returns `"true"` for true and `"false"` for false.
	fn to_text(&self, _: &Options) -> Result<Text<E>> {
		use crate::text::TextSlice;

		let true_text = unsafe { TextSlice::new_unchecked("true") };
		let false_text = unsafe { TextSlice::new_unchecked("false") };

		if *self {
			Ok(true_text.into())
		} else {
			Ok(false_text.into())
		}
	}
}
