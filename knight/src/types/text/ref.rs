use super::{TextInner, Text};
use std::borrow::Borrow;
use std::ops::Deref;

/// A reference to a [`Text`].
///
/// Due to how [`Text`] is laid out internally, some functions aren't able to return owned references.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TextRef<'a>(pub(super) &'a TextInner);

impl Deref for TextRef<'_> {
	type Target = Text;

	#[inline]
	fn deref(&self) -> &Self::Target {
		// SAFETY:
		// /*`Text` is a transparent pointer to `TextInner` whereas `TextRef` is a transparent
		// reference to the same type. Since pointers and references can be transmuted safely, this is valid.*/ <-- old
		unsafe {
			&*(self as *const TextRef<'_> as *const Text)
		}
	}
}

impl AsRef<Text> for TextRef<'_> {
	#[inline]
	fn as_ref(&self) -> &Text {
		&self
	}
}

impl Borrow<Text> for TextRef<'_> {
	#[inline]
	fn borrow(&self) -> &Text {
		&self
	}
}

impl<'a> From<&'a Text> for TextRef<'a> {
	#[inline]
	fn from(text: &'a Text) -> Self {
		Self(text.inner())
	}
}
