use super::{validate, Chars, NewTextError, Text};
use crate::env::Flags;
use crate::value::Integer;
#[cfg(feature = "extensions")]
use crate::value::{List, ToList, Value};
use std::fmt::{self, Debug, Display, Formatter};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TextSlice(str);

impl Debug for TextSlice {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl Default for &TextSlice {
	#[inline]
	fn default() -> Self {
		// SAFETY: we know that `""` is a valid string, as it contains nothing.
		unsafe { TextSlice::new_unchecked("") }
	}
}

impl Display for TextSlice {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl PartialEq<str> for TextSlice {
	fn eq(&self, rhs: &str) -> bool {
		self.0 == *rhs
	}
}

impl std::ops::Deref for TextSlice {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl TextSlice {
	/// Creates a new [`TextSlice`] without validating that `inp`'s a valid string for `E`.
	///
	/// # Safety
	/// - `inp` must be a a valid string for the encoding `E`.
	pub const unsafe fn new_unchecked(inp: &str) -> &Self {
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		&*(inp as *const str as *const Self)
	}

	/// Tries to create a new [`TextSlice`], returning an error if not possible.
	pub fn new<'s>(inp: &'s str, flags: &Flags) -> Result<&'s Self, NewTextError> {
		validate(inp, flags)?;

		Ok(unsafe { Self::new_unchecked(inp) })
	}

	#[deprecated]
	pub fn new_boxed(inp: Box<str>, flags: &Flags) -> Result<Box<Self>, NewTextError> {
		validate(&inp, flags)?;

		Ok(unsafe { Self::new_boxed_unchecked(inp) })
	}

	#[deprecated]
	pub unsafe fn new_boxed_unchecked(inp: Box<str>) -> Box<Self> {
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Box::from_raw(Box::into_raw(inp) as _)
	}

	pub const fn as_str(&self) -> &str {
		&self.0
	}

	/// Gets an iterate over [`Character`]s.
	pub fn chars(&self) -> Chars<'_> {
		Chars(self.0.chars())
	}

	pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
		let substring = self.0.get(range)?;

		// SAFETY: We're getting a substring of a valid TextSlice, which thus will itself be valid.
		Some(unsafe { Self::new_unchecked(substring) })
	}

	/// Concatenates two strings together
	pub fn concat(&self, rhs: &Self, flags: &Flags) -> Result<Text, NewTextError> {
		let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

		builder.push(self);
		builder.push(rhs);

		builder.finish(flags)
	}

	pub fn repeat(&self, amount: usize, flags: &Flags) -> Result<Text, NewTextError> {
		unsafe { Text::new_len_unchecked((**self).repeat(amount), flags) }
	}

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn split(&self, sep: &Self, env: &mut crate::env::Environment) -> List {
		if sep.is_empty() {
			// TODO: optimize me
			return Value::from(self.to_owned()).to_list(env).unwrap();
		}

		let chars = (**self)
			.split(&**sep)
			.map(|x| unsafe { Text::new_unchecked(x) }.into())
			.collect::<Vec<_>>();

		// SAFETY: If `self` is within the container bounds, so is the length of its chars.
		unsafe { List::new_unchecked(chars) }
	}

	pub fn ord(&self) -> crate::Result<Integer> {
		Integer::try_from(self.chars().next().ok_or(crate::Error::DomainError("empty string"))?)
	}

	/// Gets the first character of `self`, if it exists.
	pub fn head(&self) -> Option<char> {
		self.chars().next()
	}

	/// Gets everything _but_ the first character of `self`, if it exists.
	pub fn tail(&self) -> Option<&TextSlice> {
		self.get(1..)
	}

	pub fn remove_substr(&self, substr: &Self) -> Text {
		let _ = substr;
		todo!();
	}
}

impl<'a> From<&'a TextSlice> for &'a str {
	fn from(text: &'a TextSlice) -> Self {
		text
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
