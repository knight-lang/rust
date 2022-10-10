use crate::env::Flags;
use crate::parse::{self, Parsable, Parser};
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

impl From<Character> for Text {
	fn from(inp: Character) -> Self {
		// SAFETY: We know if we have a `Character` it's already valid, so we don't need to check for
		// validity again.
		unsafe { Self::new_unchecked(inp) }
	}
}

impl Text {
	pub fn builder() -> super::Builder {
		Default::default()
	}

	pub fn new(inp: impl ToString, flags: &Flags) -> Result<Self, NewTextError> {
		TextSlice::new_boxed(inp.to_string().into(), flags).map(|x| Self(x.into()))
	}

	pub unsafe fn new_unchecked(inp: impl ToString) -> Self {
		Self(TextSlice::new_boxed_unchecked(inp.to_string().into()).into())
	}
}

impl std::borrow::Borrow<TextSlice> for Text {
	fn borrow(&self) -> &TextSlice {
		self
	}
}

impl From<&TextSlice> for Text {
	fn from(text: &TextSlice) -> Self {
		// SAFETY: `text` is already valid.
		unsafe { Self::new_unchecked(text) }
	}
}

// impl FromIterator<Character> for Text {
// 	fn from_iter<T: IntoIterator<Item = Character>>(iter: T) -> Self {
// 		iter.into_iter().map(char::from).collect::<String>().try_into().unwrap()
// 	}
// }

impl Parsable<'_> for Text {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
		// since `.advance()` returns a `Character`, we can't match on it.
		let Some(quote) = parser.advance_if(|c| c == '\'' || c == '\"') else {
			return Ok(None);
		};

		let starting_line = parser.line();
		let body = parser.take_while(|chr, _| chr != quote).unwrap_or_default();

		if parser.advance() != Some(quote) {
			return Err(parse::ErrorKind::UnterminatedText { quote }.error(starting_line));
		}

		Ok(Some(body.to_owned()))
	}
}
