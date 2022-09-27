use super::Encoding;
use crate::value::Integer;
use crate::{Error, Result};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub struct Character<E>(char, PhantomData<E>);

impl<E> Debug for Character<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl<E> Display for Character<E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl<E> Copy for Character<E> {}
impl<E> Clone for Character<E> {
	fn clone(&self) -> Self {
		Self(self.0, self.1)
	}
}

impl<E> Eq for Character<E> {}
impl<E> PartialEq for Character<E> {
	fn eq(&self, rhs: &Self) -> bool {
		self.0 == rhs.0
	}
}

impl<E> PartialEq<char> for Character<E> {
	fn eq(&self, rhs: &char) -> bool {
		self.0 == *rhs
	}
}

impl<E> PartialOrd for Character<E> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.0.cmp(&rhs.0))
	}
}

impl<E> Ord for Character<E> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.0.cmp(&rhs.0)
	}
}

impl<E> Hash for Character<E> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}

impl<E: Encoding, I> TryFrom<Integer<I>> for Character<E> {
	type Error = Error;

	fn try_from(inp: Integer<I>) -> Result<Self> {
		u32::try_from(inp)
			.ok()
			.and_then(char::from_u32)
			.and_then(Self::new)
			.ok_or(Error::DomainError("number isn't a valid char"))
	}
}

impl<E> Character<E> {
	pub const unsafe fn new_unchecked(chr: char) -> Self {
		Self(chr, PhantomData)
	}

	pub const fn inner(self) -> char {
		self.0
	}
}

impl<E: Encoding> Character<E> {
	#[inline]
	#[must_use]
	pub fn new(chr: char) -> Option<Self> {
		if !E::is_valid(chr) {
			return None;
		}

		Some(Self(chr, PhantomData))
	}

	pub fn is_whitespace(self) -> bool {
		self.0 == ':' || E::is_whitespace(self.0)
	}

	pub fn is_numeric(self) -> bool {
		E::is_numeric(self.0)
	}

	pub fn is_lowercase(self) -> bool {
		self.0 == '_' || E::is_lowercase(self.0)
	}

	pub fn is_uppercase(self) -> bool {
		self.0 == '_' || E::is_uppercase(self.0)
	}
}

impl<E> From<Character<E>> for char {
	fn from(character: Character<E>) -> Self {
		character.0
	}
}
