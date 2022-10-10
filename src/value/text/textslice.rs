use super::{validate, Character, Chars, NewTextError, Text};
use crate::env::{Environment, Flags};
use crate::value::{Boolean, Integer, List, ToBoolean, ToInteger, ToList, ToText, Value};
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
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		&*(inp as *const str as *const Self)
	}

	pub const fn new<'s>(inp: &'s str, flags: &Flags) -> Result<&'s Self, NewTextError> {
		match validate(inp, flags) {
			// SAFETY: we justverified it was valid
			Ok(_) => Ok(unsafe { Self::new_unchecked(inp) }),

			// Can't use `?` or `Result::map` in const functions
			Err(err) => Err(err),
		}
	}

	pub fn new_boxed(inp: Box<str>, flags: &Flags) -> Result<Box<Self>, NewTextError> {
		validate(&inp, flags)?;

		#[allow(unsafe_code)]
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Ok(unsafe { Self::new_boxed_unchecked(inp) })
	}

	#[allow(unsafe_code)]
	pub unsafe fn new_boxed_unchecked(inp: Box<str>) -> Box<Self> {
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Box::from_raw(Box::into_raw(inp) as _)
	}

	pub const fn as_str(&self) -> &str {
		&self.0
	}

	pub fn chars(&self) -> Chars<'_> {
		Chars(self.0.chars())
	}

	pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
		let substring = self.0.get(range)?;

		// SAFETY: We're getting a substring of a valid TextSlice, which thus will itself be valid.
		Some(unsafe { Self::new_unchecked(substring) })
	}

	pub fn concat(&self, rhs: &Self, flags: &Flags) -> Result<Text, NewTextError> {
		let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

		builder.push(self);
		builder.push(rhs);

		builder.finish(flags)
	}

	pub fn repeat(&self, amount: usize, flags: &Flags) -> Result<Text, NewTextError> {
		Ok(Text::new((**self).repeat(amount), flags)?)
	}

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn split<'e>(&self, sep: &Self, env: &mut Environment<'e>) -> List<'e> {
		if sep.is_empty() {
			// TODO: optimize me
			return Value::from(self.to_owned()).to_list(env).unwrap();
		}

		let chars = (**self)
			.split(&**sep)
			.map(|x| Text::new(x, env.flags()).unwrap().into())
			.collect::<Vec<_>>();

		// SAFETY: If `self` is within the container bounds, so is the length of its chars.
		unsafe { List::new_unchecked(chars) }
	}

	pub fn ord(&self) -> crate::Result<Integer> {
		Integer::try_from(
			self.chars().next().ok_or(crate::Error::DomainError("empty string"))?.inner(),
		)
	}

	pub fn head(&self) -> Option<Character> {
		self.chars().next()
	}

	pub fn tail(&self) -> Option<Text> {
		let mut chrs = self.chars();

		if chrs.next().is_none() {
			None
		} else {
			Some(chrs.as_text().to_owned())
		}
	}

	pub fn remove_substr(&self, substr: &TextSlice) -> Text {
		let _ = substr;
		todo!();
	}
}

impl<'a> From<&'a TextSlice> for &'a str {
	#[inline]
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
	type Item = Character;
	type IntoIter = Chars<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.chars()
	}
}

impl<'e> ToBoolean<'e> for Text {
	fn to_boolean(&self, _: &mut Environment<'e>) -> crate::Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl<'e> ToText<'e> for Text {
	fn to_text(&self, _: &mut Environment<'e>) -> crate::Result<Self> {
		Ok(self.clone())
	}
}

impl crate::value::NamedType for Text {
	const TYPENAME: &'static str = "Text";
}

impl<'e> ToInteger<'e> for Text {
	fn to_integer(&self, _: &mut Environment<'e>) -> crate::Result<Integer> {
		self.parse()
	}
}

impl<'e> ToList<'e> for Text {
	fn to_list(&self, _: &mut Environment<'e>) -> crate::Result<List<'e>> {
		let chars = self.chars().map(Value::from).collect::<Vec<_>>();

		// SAFETY: If `self` is within the container bounds, so is the length of its chars.
		Ok(unsafe { List::new_unchecked(chars) })
	}
}
