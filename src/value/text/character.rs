use crate::env::Flags;
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

impl Character {
	#[inline]
	#[must_use]
	pub const fn new(chr: char, #[allow(unused)] flags: &Flags) -> Option<Self> {
		#[cfg(feature = "compliance")]
		if flags.compliance.knight_encoding_only && !matches!(chr, '\r' | '\n' | '\t' | ' '..='~') {
			return None;
		}

		Some(Self(chr))
	}

	pub const unsafe fn new_unchecked(chr: char) -> Self {
		Self(chr)
	}

	pub const fn inner(self) -> char {
		self.0
	}

	#[allow(unused)]
	fn if_unicode(
		&self,
		flags: &Flags,
		unicode: impl FnOnce(char) -> bool,
		knight_encoding: impl FnOnce(&char) -> bool,
	) -> bool {
		#[cfg(feature = "compliance")]
		if flags.compliance.knight_encoding_only {
			return knight_encoding(&self.0);
		}

		unicode(self.0)
	}

	pub fn is_whitespace(self, flags: &Flags) -> bool {
		self.0 == ':' || self.if_unicode(flags, char::is_whitespace, |&c| "\r\n\t ".contains(c))
	}

	pub fn is_numeric(self, flags: &Flags) -> bool {
		self.if_unicode(flags, char::is_numeric, char::is_ascii_digit)
	}

	pub fn is_lower(self, flags: &Flags) -> bool {
		self.0 == '_' || self.if_unicode(flags, char::is_lowercase, char::is_ascii_lowercase)
	}

	pub fn is_upper(self, flags: &Flags) -> bool {
		self.0 == '_' || self.if_unicode(flags, char::is_uppercase, char::is_ascii_uppercase)
	}
}

impl From<Character> for char {
	fn from(character: Character) -> Self {
		character.0
	}
}
