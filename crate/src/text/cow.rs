use super::{Text, TextRef};

#[derive(Debug)]
pub enum TextCow<'a> {
	Borrowed(TextRef<'a>),
	Owned(Text)
}

impl std::ops::Deref for TextCow<'_> {
	type Target = Text;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Borrowed(textref) => &textref,
			Self::Owned(text) => text
		}
	}
}

impl TextCow<'_> {
	pub fn into_text(self) -> Text {
		match self {
			Self::Borrowed(textref) => textref.into_owned(),
			Self::Owned(text) => text
		}
	}
}

impl From<Text> for TextCow<'_> {
	#[inline]
	fn from(text: Text) -> Self {
		Self::Owned(text)
	}
}

impl<'a> From<TextRef<'a>> for TextCow<'a> {
	#[inline]
	fn from(textref: TextRef<'a>) -> Self {
		Self::Borrowed(textref)
	}
}