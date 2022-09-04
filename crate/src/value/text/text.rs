use super::{validate, Chars, IllegalChar, SharedText};
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Text(str);

impl Default for &Text {
	#[inline]
	fn default() -> Self {
		// SAFETY: we know that `""` is a valid string, as it contains nothing.
		unsafe { Text::new_unchecked("") }
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl Deref for Text {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Text {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Text {
	/// Creates a new `Text` without validating `inp`.
	///
	/// # Safety
	/// - `inp` must be a valid `Text`.
	pub const unsafe fn new_unchecked(inp: &str) -> &Self {
		debug_assert!(validate(inp).is_ok());

		// SAFETY: Since `Text` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		&*(inp as *const str as *const Self)
	}

	pub const fn new(inp: &str) -> Result<&Self, IllegalChar> {
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

		// SAFETY: We're getting a substring of a valid Text, which thus will itself be valid.
		Some(unsafe { Self::new_unchecked(substring) })
	}

	pub fn concat(&self, rhs: &Self) -> SharedText {
		let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

		builder.push(self);
		builder.push(rhs);

		builder.finish()
	}

	pub fn repeat(&self, amount: usize) -> SharedText {
		(**self)
			.repeat(amount)
			.try_into()
			.unwrap_or_else(|_| unsafe { std::hint::unreachable_unchecked() })
	}

	#[cfg(feature = "list-extensions")]
	pub fn split(&self, sep: &Self) -> crate::List {
		if sep.is_empty() {
			// TODO: optimize me
			crate::Value::from(self.to_owned()).to_list().unwrap()
		} else {
			(**self).split(&**sep).map(|x| SharedText::new(x).unwrap().into()).collect()
		}
	}
}

impl<'a> TryFrom<&'a str> for &'a Text {
	type Error = IllegalChar;

	#[inline]
	fn try_from(inp: &'a str) -> Result<Self, Self::Error> {
		Text::new(inp)
	}
}

impl<'a> From<&'a Text> for &'a str {
	#[inline]
	fn from(text: &'a Text) -> Self {
		text
	}
}

impl TryFrom<Box<str>> for Box<Text> {
	type Error = IllegalChar;

	fn try_from(inp: Box<str>) -> Result<Self, Self::Error> {
		validate(&inp)?;

		#[allow(unsafe_code)]
		// SAFETY: Since `Text` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Ok(unsafe { Box::from_raw(Box::into_raw(inp) as _) })
	}
}

impl ToOwned for Text {
	type Owned = SharedText;

	fn to_owned(&self) -> Self::Owned {
		self.into()
	}
}

impl<'a> IntoIterator for &'a Text {
	type Item = char;
	type IntoIter = Chars<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.chars()
	}
}
