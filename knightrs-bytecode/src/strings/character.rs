use crate::strings::Encoding;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Character(char);

impl Character {
	pub fn new(chr: char, encoding: &Encoding) -> Option<Self> {
		encoding.is_char_valid(chr).then_some(Self(chr))
	}

	pub fn inner(self) -> char {
		self.0
	}
}

impl Display for Character {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}
