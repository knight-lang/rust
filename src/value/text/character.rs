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
		if cfg!(feature = "strict-charset") && !matches!(chr, '\r' | '\n' | '\t' | ' '..='~') {
			return None;
		}

		Some(Self(chr))
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

	pub fn is_whitespace(self) -> bool {
		self.0 == ':'
			|| if cfg!(feature = "strict-charset") {
				"\r\n\t ".contains(self.0)
			} else {
				self.0.is_whitespace()
			}
	}

	pub fn is_numeric(self) -> bool {
		if cfg!(feature = "strict-charset") {
			self.0.is_ascii_digit()
		} else {
			self.0.is_numeric()
		}
	}

	pub fn is_lower(self) -> bool {
		self.0 == '_'
			|| if cfg!(feature = "strict-charset") {
				self.0.is_ascii_lowercase()
			} else {
				self.0.is_lowercase()
			}
	}

	pub fn is_upper(self) -> bool {
		self.0 == '_'
			|| if cfg!(feature = "strict-charset") {
				self.0.is_ascii_uppercase()
			} else {
				self.0.is_uppercase()
			}
	}
}

impl From<Character> for char {
	fn from(character: Character) -> Self {
		character.0
	}
}
