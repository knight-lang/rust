use crate::env::{Environment, Flags};
use crate::parse::{self, Parsable, Parser};
use crate::text::{NewTextError, TextSlice};
use crate::value::{Boolean, Integer, List, NamedType, ToBoolean, ToInteger, ToList, ToText};
use crate::RefCount;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text(RefCount<TextSlice>);

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&***self, f)
	}
}

impl Default for Text {
	fn default() -> Self {
		<&TextSlice>::default().into()
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl std::ops::Deref for Text {
	type Target = TextSlice;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl PartialEq<str> for Text {
	fn eq(&self, rhs: &str) -> bool {
		**self == *rhs
	}
}

impl Text {
	pub fn builder() -> super::Builder {
		Default::default()
	}

	pub fn new<T: ToString>(inp: T, flags: &Flags) -> Result<Self, NewTextError> {
		TextSlice::new_boxed(inp.to_string().into(), flags).map(|x| Self(x.into()))
	}

	pub unsafe fn new_unchecked<T: ToString>(inp: T) -> Self {
		let boxed = inp.to_string().into_boxed_str();

		Self(RefCount::from(Box::from_raw(Box::into_raw(boxed) as *mut TextSlice)))
	}

	pub unsafe fn new_len_unchecked<T>(inp: T, flags: &Flags) -> Result<Self, NewTextError>
	where
		T: ToString,
	{
		let inp = inp.to_string();
		super::validate_len(&inp, flags)?;

		Ok(Self::new_unchecked(inp))
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

impl Parsable for Text {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
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

impl NamedType for Text {
	const TYPENAME: &'static str = "Text";
}

impl ToBoolean for Text {
	#[inline]
	fn to_boolean(&self, _: &mut Environment) -> crate::Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToText for Text {
	#[inline]
	fn to_text(&self, _: &mut Environment) -> crate::Result<Self> {
		Ok(self.clone())
	}
}

impl ToInteger for Text {
	#[inline]
	fn to_integer(&self, _: &mut Environment) -> crate::Result<Integer> {
		Ok(self.parse().unwrap_or_default())
	}
}

impl ToList for Text {
	fn to_list(&self, _: &mut Environment) -> crate::Result<List> {
		let chars =
			self.chars().map(|c| unsafe { Self::new_unchecked(c) }.into()).collect::<Vec<_>>();

		// SAFETY: If `self` is within the container bounds, so is the length of its chars.
		Ok(unsafe { List::new_unchecked(chars) })
	}
}
