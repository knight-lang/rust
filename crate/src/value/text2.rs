#![allow(unused)]
use crate::value::{List, ToList, Value};
use crate::{Error, RefCount};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text<'a>(Inner<'a>);

// We can't expose this enum because then people could make arbitrary `Slice`s.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Inner<'a> {
	Slice(&'a str),
	Owned(RefCount<str>),
}

impl Debug for Text<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_tuple("Text").field(&self.as_str()).finish()
	}
}

impl Default for Text<'_> {
	#[inline]
	fn default() -> Self {
		// SAFETY: we know that `""` is a valid string, as it contains nothing.
		const EMPTY: Text<'static> = unsafe { Text::new_unchecked("") };

		EMPTY
	}
}

impl Display for Text<'_> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(self.as_str(), f)
	}
}

/// An error that indicates there was a problem with a string.
#[derive(Debug, PartialEq, Eq)]
pub enum IllegalString {
	/// The length of the input was too large.
	///
	/// This is only returned when the `container-length-limit` feature is enabled.
	TooLong(usize),

	/// A character within a string wasn't a valid Knight character.
	///
	/// This is only returned when the `strict-charset` feature is enabled.
	BadChar {
		/// The char that was invalid.
		chr: char,

		/// The index of the invalid char in the given string.
		idx: usize,
	},
}

impl Display for IllegalString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::TooLong(len) => write!(f, "length {len} is too large ({} max)", Text::MAX_LEN),
			Self::BadChar { chr, idx } => write!(f, "illegal char {chr:?} found at {idx:?}"),
		}
	}
}

impl std::error::Error for IllegalString {}

const fn validate(data: &str) -> Result<(), IllegalString> {
	if cfg!(feature = "container-length-limit") && Text::MAX_LEN < data.len() {
		return Err(IllegalString::TooLong(data.len()));
	}

	// All characters are valid under normal mode.
	if cfg!(not(feature = "strict-charset")) {
		return Ok(());
	}

	// We're in const context, so we must use `while` with bytes.
	// Since we're not using unicode, everything's just a byte anyways.
	let bytes = data.as_bytes();
	let mut idx = 0;

	while idx < bytes.len() {
		let chr = bytes[idx] as char;

		if !matches!(chr, '\r' | '\n' | '\t' | ' '..='~') {
			// Since everything's a byte, the byte index is the same as the char index.
			return Err(IllegalString::BadChar { chr, idx });
		}

		idx += 1;
	}

	Ok(())
}

impl<'a> TryFrom<&'a str> for Text<'a> {
	type Error = IllegalString;

	fn try_from(inp: &'a str) -> Result<Self, Self::Error> {
		validate(inp).map(|_| Self(Inner::Slice(inp)))
	}
}

impl TryFrom<Box<str>> for Text<'_> {
	type Error = IllegalString;

	fn try_from(inp: Box<str>) -> Result<Self, Self::Error> {
		validate(&inp).map(|_| Self(Inner::Owned(inp.into())))
	}
}

impl TryFrom<String> for Text<'_> {
	type Error = IllegalString;

	fn try_from(inp: String) -> Result<Self, Self::Error> {
		inp.into_boxed_str().try_into()
	}
}

impl<'a> Text<'a> {
	/// The maximum length for [`Text`]s. Only used when `container-length-limit` is enabled.
	pub const MAX_LEN: usize = i32::MAX as usize;

	/// Creates a new `Text` without validating `inp`.
	///
	/// # Safety
	/// - `inp` must be a valid `Text`.
	#[inline]
	pub const unsafe fn new_unchecked(inp: &'a str) -> Self {
		debug_assert!(validate(inp).is_ok());

		Self(Inner::Slice(inp))
	}

	pub const fn new(inp: &'a str) -> Result<Self, IllegalString> {
		match validate(inp) {
			// SAFETY: we justverified it was valid
			Ok(_) => Ok(unsafe { Self::new_unchecked(inp) }),

			// Can't use `?` or `Result::map` in const functions
			Err(err) => Err(err),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self.0 {
			Inner::Slice(slice) => slice.is_empty(),
			Inner::Owned(ref rc) => rc.is_empty(),
		}
	}

	pub fn len(&self) -> usize {
		match self.0 {
			Inner::Slice(slice) => slice.len(),
			Inner::Owned(ref rc) => rc.len(),
		}
	}

	pub fn to_owned(&self) -> Text<'static> {
		match self.0 {
			Inner::Slice(slice) => slice.to_string().try_into().unwrap(),
			Inner::Owned(ref rc) => Text(Inner::Owned(rc.clone())),
		}
	}

	pub fn as_str(&self) -> &str {
		match self.0 {
			Inner::Slice(slice) => slice,
			Inner::Owned(ref rc) => rc.as_ref(),
		}
	}

	pub fn chars(&self) -> Chars<'_> {
		Chars(self.as_str().chars())
	}

	pub fn get<T>(&self, range: T) -> Option<Text<'_>>
	where
		T: std::slice::SliceIndex<str, Output = str>,
	{
		self.as_str().get(range).map(|substr| substr.try_into().unwrap())
	}

	pub fn concat(&self, rhs: &Text<'_>) -> Text<'static> {
		// FIXME: use an actual builder?
		let mut string = String::with_capacity(self.len() + rhs.len());

		string.push_str(self.as_str());
		string.push_str(rhs.as_str());

		string.try_into().unwrap()
	}

	pub fn repeat(&self, amount: usize) -> Result<Text<'static>, IllegalString> {
		if amount == 0 {
			return Ok(Text::default());
		}

		// if cfg!(feature = "container-length-limit") && self.len().checked_mul(amount).map_or(false, |len| Self::MAX_LEN < len) {
		// 	return Err(IllegalString::TooLarge)
		// }

		self.as_str().repeat(amount).try_into()
	}

	pub fn split<'e>(&self, sep: &Text<'_>) -> List<'e> {
		if sep.is_empty() {
			// return self.to_list().unwrap();
		}

		todo!();
		// self
		// 	.as_str()
		// 	.split(sep.as_str())
		// 	.map(|s| unsafe { Text::from_string_unchecked(s.into()) })
		// 	.map(Value::from)
		// 	.collect::<Vec<_>>()
		// 	.try_into()
		// 	.unwrap() // this won't fail, as string length <= max list length.
	}
}

impl<'a> IntoIterator for &'a Text<'_> {
	type Item = char;
	type IntoIter = Chars<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.chars()
	}
}

pub struct Chars<'a>(std::str::Chars<'a>);

impl<'a> Chars<'a> {
	pub fn as_text(&self) -> Text<'a> {
		self.0.as_str().try_into().unwrap()
	}
}

impl Iterator for Chars<'_> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}
