mod builder;
mod character;
mod encoding;
mod text;
mod textslice;

pub use encoding::*;

pub trait ToText<E> {
	fn to_text(&self, _: &crate::env::Options) -> crate::Result<Text<E>>;
}

pub use builder::Builder;
pub use character::Character;
pub use text::*;
pub use textslice::*;

pub struct Chars<'a, E>(std::str::Chars<'a>, std::marker::PhantomData<E>);
impl<'a, E> Chars<'a, E> {
	pub fn as_text(&self) -> &'a TextSlice<E> {
		unsafe { TextSlice::new_unchecked(self.0.as_str()) }
	}
}

impl<E> Iterator for Chars<'_, E> {
	type Item = Character<E>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|chr| unsafe { Character::new_unchecked(chr) })
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum NewTextError {
	/// Indicates a Knight string was too long.
	LengthTooLong(usize),

	/// Indicates a character within a Knight string wasn't valid.
	IllegalChar {
		/// The char that was invalid.
		chr: char,

		/// The index of the invalid char in the given string.
		index: usize,
	},
}

impl std::fmt::Display for NewTextError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::LengthTooLong(len) => {
				write!(f, "length {len } longer than max {}", TextSlice::<u8>::MAX_LEN)
			}
			Self::IllegalChar { chr, index } => {
				write!(f, "illegal char {chr:?} found at index {index}")
			}
		}
	}
}

impl std::error::Error for NewTextError {}

pub fn validate<E: Encoding>(data: &str) -> Result<(), NewTextError> {
	if cfg!(feature = "container-length-limit") && TextSlice::<E>::MAX_LEN < data.len() {
		return Err(NewTextError::LengthTooLong(data.len()));
	}

	E::validate_contents(data)
}
