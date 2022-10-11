use super::Encoding;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A single character of a specific encoding.
pub struct Character<E>(char, PhantomData<E>);

impl<E> Copy for Character<E> {}
impl<E> Clone for Character<E> {
	fn clone(&self) -> Self {
		*self
	}
}

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
		self.0.partial_cmp(&rhs.0)
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

impl<E> Character<E> {
	/// Creates a new [`Character`] without ensuring that it's a valid `E`.
	///
	/// # Safety
	/// Callers must ensure that `chr` is actually a valid character for the encoding `E`.
	#[must_use]
	pub const unsafe fn new_unchecked(chr: char) -> Self {
		Self(chr, PhantomData)
	}

	/// Gets the wrapped `char`.
	#[must_use]
	pub const fn inner(self) -> char {
		self.0
	}
}

impl<E: Encoding> Character<E> {
	/// Creates a new [`Character`], returning `None` if `chr` is not valid for the encoding.
	#[must_use]
	pub fn new(chr: char) -> Option<Self> {
		E::is_valid(chr).then_some(Self(chr, PhantomData))
	}

	/// Checks to see if `self` is a whitespace character.
	#[must_use]
	pub fn is_whitespace(self) -> bool {
		E::is_whitespace(self.0)
	}

	/// Checks to see if `self` is a numeric character.
	#[must_use]
	pub fn is_numeric(self) -> bool {
		E::is_numeric(self.0)
	}

	/// Checks to see if `self` is a lowercase or `_` character.
	#[must_use]
	pub fn is_lower(self) -> bool {
		self.0 == '_' || E::is_lower(self.0)
	}

	/// Checks to see if `self` is an uppercase or `_` character.
	#[must_use]
	pub fn is_upper(self) -> bool {
		self.0 == '_' || E::is_upper(self.0)
	}
}

impl<E> From<Character<E>> for char {
	fn from(character: Character<E>) -> Self {
		character.0
	}
}
