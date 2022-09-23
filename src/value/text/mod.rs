mod builder;
mod character;
mod encoding;
mod text;
mod textslice;

pub use encoding::*;

pub trait ToText {
	fn to_text(&self, _: &crate::env::Options) -> crate::Result<Text>;
}

pub use builder::Builder;
pub use character::Character;
pub use text::*;
pub use textslice::*;

pub struct Chars<'a>(std::str::Chars<'a>);
impl<'a> Chars<'a> {
	pub fn as_text(&self) -> &'a TextSlice {
		unsafe { TextSlice::new_unchecked(self.0.as_str()) }
	}
}

impl Iterator for Chars<'_> {
	type Item = Character;

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
				write!(f, "length {len } longer than max {}", TextSlice::MAX_LEN)
			}
			Self::IllegalChar { chr, index } => {
				write!(f, "illegal char {chr:?} found at index {index}")
			}
		}
	}
}

impl std::error::Error for NewTextError {}

pub const fn validate<E: Encoding>(data: &str) -> Result<(), NewTextError> {
	if cfg!(feature = "container-length-limit") && TextSlice::MAX_LEN < data.len() {
		return Err(NewTextError::LengthTooLong(data.len()));
	}

	// All valid `str`s are valid TextSlice when no length limit and no char requirements are set.
	if cfg!(not(feature = "strict-charset")) {
		return Ok(());
	}

	// We're in const context, so we must use `while` with bytes.
	// Since we're not using unicode, everything's just a byte anyways.
	let bytes = data.as_bytes();
	let mut index = 0;

	while index < bytes.len() {
		let chr = bytes[index] as char;

		if Character::<E>::new(chr).is_none() {
			// Since everything's a byte, the byte index is the same as the char index.
			return Err(NewTextError::IllegalChar { chr, index });
		}

		index += 1;
	}

	Ok(())
}
