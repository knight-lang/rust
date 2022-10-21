use super::Encoding;
use crate::env::Flags;
use crate::parse::{self, Parsable, Parser};
use crate::text::{NewTextError, TextSlice};
use crate::RefCount;
use std::fmt::{self, Debug, Display, Formatter};

#[derive_where(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text<E>(RefCount<TextSlice<E>>);

impl<E> Debug for Text<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&***self, f)
	}
}

impl<E> Default for Text<E> {
	fn default() -> Self {
		<&TextSlice<E>>::default().into()
	}
}

impl<E> Display for Text<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl<E> std::ops::Deref for Text<E> {
	type Target = TextSlice<E>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<E> PartialEq<str> for Text<E> {
	fn eq(&self, rhs: &str) -> bool {
		**self == *rhs
	}
}

impl<E> Text<E> {
	pub fn builder() -> super::Builder<E> {
		Default::default()
	}

	pub fn new<I>(inp: I, flags: &Flags) -> Result<Self, NewTextError>
	where
		I: ToString,
		E: Encoding,
	{
		TextSlice::new_boxed(inp.to_string().into(), flags).map(|x| Self(x.into()))
	}

	pub unsafe fn new_unchecked<I>(inp: I) -> Self
	where
		I: ToString,
	{
		let boxed = inp.to_string().into_boxed_str();

		Self(RefCount::from(Box::from_raw(Box::into_raw(boxed) as *mut TextSlice<E>)))
	}

	pub unsafe fn new_len_unchecked<I>(inp: I, flags: &Flags) -> Result<Self, NewTextError>
	where
		I: ToString,
	{
		let inp = inp.to_string();
		super::validate_len::<E>(&inp, flags)?;

		Ok(Self::new_unchecked(inp))
	}
}

impl<E> std::borrow::Borrow<TextSlice<E>> for Text<E> {
	fn borrow(&self) -> &TextSlice<E> {
		self
	}
}

impl<E> From<&TextSlice<E>> for Text<E> {
	fn from(text: &TextSlice<E>) -> Self {
		// SAFETY: `text` is already valid.
		unsafe { Self::new_unchecked(text) }
	}
}

// impl FromIterator<Character> for Text {
// 	fn from_iter<T: IntoIterator<Item = Character>>(iter: T) -> Self {
// 		iter.into_iter().map(char::from).collect::<String>().try_into().unwrap()
// 	}
// }

impl<I, E> Parsable<I, E> for Text<E> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> parse::Result<Option<Self>> {
		// since `.advance()` returns a `Character`, we can't match on it.
		let Some(quote) = parser.advance_if(|c| c == '\'' || c == '\"') else {
			return Ok(None);
		};

		let starting_line = parser.line();
		let body = parser.take_while(|chr| chr != quote).unwrap_or_default();

		if parser.advance() != Some(quote) {
			return Err(parse::ErrorKind::UnterminatedText { quote }.error(starting_line));
		}

		Ok(Some(body.to_owned()))
	}
}
