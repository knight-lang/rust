use crate::text::{IllegalChar, Text};
use crate::Integer;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharedText(crate::RefCount<Text>);

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(SharedText: Send, Sync);

impl Default for SharedText {
	#[inline]
	fn default() -> Self {
		<&Text>::default().into()
	}
}

impl Display for SharedText {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl std::ops::Deref for SharedText {
	type Target = Text;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<Box<Text>> for SharedText {
	#[inline]
	fn from(text: Box<Text>) -> Self {
		Self(text.into())
	}
}

impl TryFrom<String> for SharedText {
	type Error = IllegalChar;

	fn try_from(inp: String) -> Result<Self, Self::Error> {
		let boxed = Box::<Text>::try_from(inp.into_boxed_str())?;

		Ok(Self(boxed.into()))
	}
}

impl SharedText {
	pub fn builder() -> super::Builder {
		Default::default()
	}

	pub fn new(inp: impl ToString) -> Result<Self, IllegalChar> {
		inp.to_string().try_into()
	}

	pub fn to_integer(&self) -> crate::Result<Integer> {
		self.parse()
	}
}

impl std::borrow::Borrow<Text> for SharedText {
	fn borrow(&self) -> &Text {
		self
	}
}

impl From<&Text> for SharedText {
	fn from(text: &Text) -> Self {
		Box::<Text>::try_from(text.to_string().into_boxed_str()).unwrap().into()
	}
}

impl TryFrom<&str> for SharedText {
	type Error = IllegalChar;

	#[inline]
	fn try_from(inp: &str) -> Result<Self, Self::Error> {
		<&Text>::try_from(inp).map(From::from)
	}
}
