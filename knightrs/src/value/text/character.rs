use crate::env::Flags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Character(char);

impl PartialEq<char> for Character {
	fn eq(&self, rhs: &char) -> bool {
		self.0 == *rhs
	}
}

impl Character {
	pub const fn new(chr: char, flags: &Flags) -> Option<Self> {
		if !super::is_valid_character(chr, flags) {
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
}

impl AsRef<char> for Character {
	fn as_ref(&self) -> &char {
		&self.0
	}
}

impl std::borrow::Borrow<char> for Character {
	fn borrow(&self) -> &char {
		&self.0
	}
}
