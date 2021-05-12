use super::{Inner, Text, TextCow, ToText};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRef<'a>(NonNull<Inner>, PhantomData<&'a ()>);

assert_eq_size!(TextRef, Text);
assert_eq_align!(TextRef, Text);

impl<'a> TextRef<'a> {
	#[inline]
	pub fn new(text: &'a Text) -> Self {
		Self(text.0, PhantomData)
	}

	pub(crate) unsafe fn from_raw(raw: *const ()) -> Self {
		Self(Text::from_raw(raw).0, PhantomData)
	}

	pub fn as_text(&self) -> &'a Text {
		unsafe {
			&*(self as *const Self as *const Text)
		}
	}

	pub(crate) fn into_owned(self) -> Text {
		(*self).clone()
	}
}

impl std::ops::Deref for TextRef<'_> {
	type Target = Text;

	fn deref(&self) -> &Text {
		self.as_text()
	}
}

impl std::borrow::Borrow<Text> for TextRef<'_> {
	fn borrow(&self) -> &Text {
		&self
	}
}

impl AsRef<Text> for TextRef<'_> {
	fn as_ref(&self) -> &Text {
		&self
	}
}

impl ToText for TextRef<'_> {
	fn to_text(&self) -> crate::Result<TextCow<'_>> {
		Ok(TextCow::Borrowed(*self))
	}
}
