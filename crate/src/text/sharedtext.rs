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
		let mut bytes = self.trim_start().bytes();

		let (is_negative, mut number) = match bytes.next() {
			Some(b'+') => (false, 0),
			Some(b'-') => (true, 0),
			Some(num @ b'0'..=b'9') => (false, (num - b'0') as Integer),
			_ => return Ok(0),
		};

		while let Some(digit @ b'0'..=b'9') = bytes.next() {
			#[cfg(feature = "checked-overflow")]
			{
				number = number
					.checked_mul(10)
					.and_then(|num| num.checked_add((digit as u8 - b'0') as _))
					.ok_or(Error::IntegerOverflow)?;
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				number = (number * 10) + (digit - b'0') as Integer;
			}
		}

		if is_negative {
			#[cfg(feature = "checked-overflow")]
			{
				number = number.checked_neg().ok_or(Error::IntegerOverflow)?;
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				number = -number;
			}
		}

		Ok(number)
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
