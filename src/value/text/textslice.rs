use super::{validate, Chars, NewTextError, Text};
use crate::value::{ToBoolean, ToInteger, ToList, ToText};
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TextSlice(str);

impl Default for &TextSlice {
	#[inline]
	fn default() -> Self {
		// SAFETY: we know that `""` is a valid string, as it contains nothing.
		unsafe { TextSlice::new_unchecked("") }
	}
}

impl Display for TextSlice {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl Deref for TextSlice {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for TextSlice {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl TextSlice {
	pub const MAX_LEN: usize = i32::MAX as usize;

	/// Creates a new `TextSlice` without validating `inp`.
	///
	/// # Safety
	/// - `inp` must be a valid `TextSlice`.
	pub const unsafe fn new_unchecked(inp: &str) -> &Self {
		debug_assert!(validate(inp).is_ok());

		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		&*(inp as *const str as *const Self)
	}

	pub const fn new(inp: &str) -> Result<&Self, NewTextError> {
		match validate(inp) {
			// SAFETY: we justverified it was valid
			Ok(_) => Ok(unsafe { Self::new_unchecked(inp) }),

			// Can't use `?` or `Result::map` in const functions
			Err(err) => Err(err),
		}
	}

	pub fn chars(&self) -> Chars<'_> {
		Chars(self.0.chars())
	}

	pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
		let substring = self.0.get(range)?;

		// SAFETY: We're getting a substring of a valid TextSlice, which thus will itself be valid.
		Some(unsafe { Self::new_unchecked(substring) })
	}

	pub fn concat(&self, rhs: &Self) -> Text {
		let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

		builder.push(self);
		builder.push(rhs);

		builder.finish()
	}

	pub fn repeat(&self, amount: usize) -> Text {
		(**self)
			.repeat(amount)
			.try_into()
			.unwrap_or_else(|_| unsafe { std::hint::unreachable_unchecked() })
	}

	#[cfg(feature = "list-extensions")]
	pub fn split<'e>(&self, sep: &Self) -> crate::List<'e> {
		if sep.is_empty() {
			// TODO: optimize me
			crate::Value::from(self.to_owned()).to_list().unwrap()
		} else {
			(**self)
				.split(&**sep)
				.map(|x| Text::new(x).unwrap().into())
				.collect::<Vec<_>>()
				.try_into()
				.unwrap()
		}
	}
}

impl<'a> TryFrom<&'a str> for &'a TextSlice {
	type Error = NewTextError;

	#[inline]
	fn try_from(inp: &'a str) -> Result<Self, Self::Error> {
		TextSlice::new(inp)
	}
}

impl<'a> From<&'a TextSlice> for &'a str {
	#[inline]
	fn from(text: &'a TextSlice) -> Self {
		text
	}
}

impl TryFrom<Box<str>> for Box<TextSlice> {
	type Error = NewTextError;

	fn try_from(inp: Box<str>) -> Result<Self, Self::Error> {
		validate(&inp)?;

		#[allow(unsafe_code)]
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Ok(unsafe { Box::from_raw(Box::into_raw(inp) as _) })
	}
}

impl ToOwned for TextSlice {
	type Owned = Text;

	fn to_owned(&self) -> Self::Owned {
		self.into()
	}
}

impl<'a> IntoIterator for &'a TextSlice {
	type Item = char;
	type IntoIter = Chars<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.chars()
	}
}

impl ToBoolean for Text {
	fn to_boolean(&self) -> crate::Result<crate::Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToText for Text {
	fn to_text(&self) -> crate::Result<Self> {
		Ok(self.clone())
	}
}

impl crate::value::NamedType for Text {
	const TYPENAME: &'static str = "Text";
}

impl ToInteger for Text {
	fn to_integer(&self) -> crate::Result<crate::value::Integer> {
		self.parse()
	}
}

impl<'e> ToList<'e> for Text {
	fn to_list(&self) -> crate::Result<crate::value::List<'e>> {
		self
			.chars()
			.map(|c| crate::Value::from(Self::try_from(c.to_string()).unwrap()))
			.collect::<Vec<_>>()
			.try_into()
	}
}
