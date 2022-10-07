use crate::text::{Character, NewTextError, TextSlice};
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text(crate::RefCount<TextSlice>);

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Text: Send, Sync);

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&***self, f)
	}
}

impl Default for Text {
	#[inline]
	fn default() -> Self {
		<&TextSlice>::default().into()
	}
}

impl Display for Text {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl std::ops::Deref for Text {
	type Target = TextSlice;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<Box<TextSlice>> for Text {
	#[inline]
	fn from(text: Box<TextSlice>) -> Self {
		Self(text.into())
	}
}

impl TryFrom<String> for Text {
	type Error = NewTextError;

	fn try_from(inp: String) -> Result<Self, Self::Error> {
		let boxed = Box::<TextSlice>::try_from(inp.into_boxed_str())?;

		Ok(Self(boxed.into()))
	}
}

impl From<Character> for Text {
	fn from(inp: Character) -> Self {
		Self::new(inp).unwrap()
	}
}

impl Text {
	pub fn builder() -> super::Builder {
		Default::default()
	}

	pub fn new(inp: impl ToString) -> Result<Self, NewTextError> {
		inp.to_string().try_into()
	}
}

impl std::borrow::Borrow<TextSlice> for Text {
	fn borrow(&self) -> &TextSlice {
		self
	}
}

impl From<&TextSlice> for Text {
	fn from(text: &TextSlice) -> Self {
		Box::<TextSlice>::try_from(text.to_string().into_boxed_str()).unwrap().into()
	}
}

impl TryFrom<&str> for Text {
	type Error = NewTextError;

	#[inline]
	fn try_from(inp: &str) -> Result<Self, Self::Error> {
		<&TextSlice>::try_from(inp).map(From::from)
	}
}

impl FromIterator<Character> for Text {
	fn from_iter<T: IntoIterator<Item = Character>>(iter: T) -> Self {
		iter.into_iter().map(char::from).collect::<String>().try_into().unwrap()
	}
}
