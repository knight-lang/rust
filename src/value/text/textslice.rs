use super::{validate, Character, Chars, Encoding, NewTextError, Text};
use crate::env::Options;
use crate::value::{Integer, List, ToBoolean, ToInteger, ToList, ToText};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[repr(transparent)]
pub struct TextSlice<E>(PhantomData<E>, str);

impl<E> Debug for TextSlice<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&**self, f)
	}
}

impl<E> Eq for TextSlice<E> {}
impl<E> PartialEq for TextSlice<E> {
	fn eq(&self, rhs: &Self) -> bool {
		**self == **rhs
	}
}

impl<E> PartialOrd for TextSlice<E> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}
impl<E> Ord for TextSlice<E> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		(**self).cmp(&**rhs)
	}
}
impl<E> Hash for TextSlice<E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(**self).hash(state)
	}
}

impl<E> Default for &TextSlice<E> {
	#[inline]
	fn default() -> Self {
		// SAFETY: we know that `""` is a valid string, as it contains nothing.
		unsafe { TextSlice::new_unchecked("") }
	}
}

impl<E> Display for TextSlice<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl<E> std::ops::Deref for TextSlice<E> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.1
	}
}

impl<E: Encoding> TextSlice<E> {
	pub fn new(inp: &str) -> Result<&Self, NewTextError> {
		match validate::<E>(inp) {
			// SAFETY: we justverified it was valid
			Ok(_) => Ok(unsafe { Self::new_unchecked(inp) }),

			// Can't use `?` or `Result::map` in const functions
			Err(err) => Err(err),
		}
	}
}

impl<E> TextSlice<E> {
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

	pub unsafe fn from_boxed_unchecked(inp: Box<str>) -> Box<Self> {
		// SAFETY: Since `TextSlice` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		unsafe { Box::from_raw(Box::into_raw(inp) as _) }
	}

	pub fn chars(&self) -> Chars<'_, E> {
		Chars(self.1.chars(), PhantomData)
	}

	pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
		let substring = self.1.get(range)?;

		// SAFETY: We're getting a substring of a valid TextSlice, which thus will itself be valid.
		Some(unsafe { Self::new_unchecked(substring) })
	}

	pub fn concat(&self, rhs: &Self) -> Text<E> {
		let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

		builder.push(self);
		builder.push(rhs);

		builder.finish()
	}

	pub fn repeat(&self, amount: usize) -> Text<E> {
		let repeated = (**self).repeat(amount);

		unsafe { Text::new_unchecked(repeated) }
	}

	pub fn split<'e, I>(&self, sep: &Self, opts: &Options) -> List<'e, E, I> {
		if sep.is_empty() {
			// TODO: optimize me
			crate::Value::from(self.to_owned()).to_list(opts).unwrap()
		} else {
			(**self)
				.split(&**sep)
				.map(|x| unsafe { Text::new_unchecked(x.to_string()).into() })
				.collect::<Vec<_>>()
				.try_into()
				.unwrap()
		}
	}

	pub fn ord<I>(&self) -> crate::Result<Integer<I>> {
		Integer::try_from(
			self.chars().next().ok_or(crate::Error::DomainError("empty string"))?.inner(),
		)
	}

	pub fn head(&self) -> Option<Character<E>> {
		self.chars().next()
	}

	pub fn tail(&self) -> Option<Text<E>> {
		let mut chrs = self.chars();

		if chrs.next().is_none() {
			None
		} else {
			Some(chrs.as_text().to_owned())
		}
	}
}

impl<'a, E: Encoding> TryFrom<&'a str> for &'a TextSlice<E> {
	type Error = NewTextError;

	#[inline]
	fn try_from(inp: &'a str) -> Result<Self, Self::Error> {
		TextSlice::new(inp)
	}
}

impl<'a, E> From<&'a TextSlice<E>> for &'a str {
	#[inline]
	fn from(text: &'a TextSlice<E>) -> Self {
		text
	}
}

impl<E: Encoding> TryFrom<Box<str>> for Box<TextSlice<E>> {
	type Error = NewTextError;

	fn try_from(inp: Box<str>) -> Result<Self, Self::Error> {
		validate::<E>(&inp)?;

		Ok(unsafe { TextSlice::from_boxed_unchecked(inp) })
	}
}

impl<E> ToOwned for TextSlice<E> {
	type Owned = Text<E>;

	fn to_owned(&self) -> Self::Owned {
		self.into()
	}
}

impl<'a, E> IntoIterator for &'a TextSlice<E> {
	type Item = Character<E>;
	type IntoIter = Chars<'a, E>;

	fn into_iter(self) -> Self::IntoIter {
		self.chars()
	}
}

impl<E> ToBoolean for Text<E> {
	fn to_boolean(&self, _: &Options) -> crate::Result<crate::Boolean> {
		Ok(!self.is_empty())
	}
}

impl<E> ToText<E> for Text<E> {
	fn to_text(&self, _: &Options) -> crate::Result<Self> {
		Ok(self.clone())
	}
}

impl<E> crate::value::NamedType for Text<E> {
	const TYPENAME: &'static str = "Text";
}

impl<E, I: crate::value::integer::IntType> ToInteger<I> for Text<E> {
	fn to_integer(&self, opts: &Options) -> crate::Result<Integer<I>> {
		Integer::parse(&self, opts)
	}
}

impl<'e, E, I> ToList<'e, E, I> for Text<E> {
	fn to_list(&self, _: &Options) -> crate::Result<List<'e, E, I>> {
		self.chars().map(|c| Self::from(c).into()).collect::<Vec<_>>().try_into()
	}
}
