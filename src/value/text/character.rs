use crate::{Error, Result};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Character(char);

impl Display for Character {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl PartialEq<char> for Character {
	fn eq(&self, rhs: &char) -> bool {
		self.0 == *rhs
	}
}

impl TryFrom<crate::value::Integer> for Character {
	type Error = Error;

	fn try_from(inp: crate::value::Integer) -> Result<Self> {
		u32::try_from(inp)
			.ok()
			.and_then(char::from_u32)
			.and_then(Character::new)
			.ok_or(Error::DomainError("number isn't a valid char"))
	}
}

impl Character {
	#[inline]
	#[must_use]
	pub const fn new(chr: char) -> Option<Self> {
		if !cfg!(feature = "knight-encoding") || matches!(chr, '\r' | '\n' | '\t' | ' '..='~') {
			Some(Self(chr))
		} else {
			None
		}
	}

	pub const unsafe fn new_unchecked(chr: char) -> Self {
		match Self::new(chr) {
			Some(character) => character,
			None => unreachable!(),
		}
	}

	pub const fn inner(self) -> char {
		self.0
	}

	fn if_unicode(
		&self,
		unicode: impl FnOnce(char) -> bool,
		knight_encoding: impl FnOnce(&char) -> bool,
	) -> bool {
		if cfg!(feature = "knight-encoding") {
			knight_encoding(&self.0)
		} else {
			unicode(self.0)
		}
	}

	pub fn is_whitespace(self) -> bool {
		self.0 == ':' || self.if_unicode(char::is_whitespace, |&c| "\r\n\t".contains(c))
	}

	pub fn is_numeric(self) -> bool {
		self.if_unicode(char::is_numeric, char::is_ascii_digit)
	}

	pub fn is_lower(self) -> bool {
		self.0 == '_' || self.if_unicode(char::is_lowercase, char::is_ascii_lowercase)
	}

	pub fn is_upper(self) -> bool {
		self.0 == '_' || self.if_unicode(char::is_uppercase, char::is_ascii_uppercase)
	}
}

impl From<Character> for char {
	fn from(character: Character) -> Self {
		character.0
	}
}
