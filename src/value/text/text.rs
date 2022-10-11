use super::Encoding;
use crate::env::Flags;
use crate::parse::{self, Parsable, Parser};
use crate::text::{Character, NewTextError, TextSlice};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

pub struct Text<E>(crate::RefCount<TextSlice<E>>);

impl<E> Clone for Text<E> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<E> Eq for Text<E> {}
impl<E> PartialEq for Text<E> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0 == rhs.0
	}
}
impl<E> PartialOrd for Text<E> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&rhs.0)
	}
}
impl<E> Ord for Text<E> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.0.cmp(&rhs.0)
	}
}
impl<E> Hash for Text<E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Text: Send, Sync);

impl<E> Debug for Text<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&***self, f)
	}
}

impl<E> Default for Text<E> {
	#[inline]
	fn default() -> Self {
		<&TextSlice<E>>::default().into()
	}
}

impl<E> Display for Text<E> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl<E> std::ops::Deref for Text<E> {
	type Target = TextSlice<E>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<E> From<Character<E>> for Text<E> {
	fn from(inp: Character<E>) -> Self {
		// SAFETY: We know if we have a `Character` it's already valid, so we don't need to check for
		// validity again.
		unsafe { Self::new_unchecked(inp) }
	}
}

impl<E> Text<E> {
	pub fn builder() -> super::Builder<E>
	where
		E: Encoding,
	{
		Default::default()
	}

	pub fn new(inp: impl ToString, flags: &Flags) -> Result<Self, NewTextError>
	where
		E: Encoding,
	{
		TextSlice::new_boxed(inp.to_string().into(), flags).map(|x| Self(x.into()))
	}

	pub unsafe fn new_unchecked(inp: impl ToString) -> Self {
		Self(TextSlice::new_boxed_unchecked(inp.to_string().into()).into())
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

impl<I: crate::value::IntType, E: Encoding> Parsable<'_, I, E> for Text<E> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> parse::Result<Option<Self>> {
		// since `.advance()` returns a `Character`, we can't match on it.
		let Some(quote) = parser.advance_if(|c| c == '\'' || c == '\"') else {
			return Ok(None);
		};

		let starting_line = parser.line();
		let body = parser.take_while(|chr| chr != quote).unwrap_or_default();

		if parser.advance() != Some(quote) {
			return Err(
				parse::ErrorKind::UnterminatedText { quote: quote.inner() }.error(starting_line),
			);
		}

		Ok(Some(body.to_owned()))
	}
}
