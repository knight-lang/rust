mod builder;
// mod encoding;
mod text;
mod textslice;

pub trait ToText<I, E> {
	fn to_text(&self, env: &mut crate::Environment<I, E>) -> crate::Result<Text<E>>;
}

use crate::env::Flags;
pub use builder::Builder;
// pub use encoding::*;
pub use text::*;
pub use textslice::*;

pub const fn is_valid_character(chr: char) -> bool {
	matches!(chr, '\r' | '\n' | '\t' | ' '..='~')
}

pub struct Chars<'a, E>(std::marker::PhantomData<E>, std::str::Chars<'a>);
impl<'a, E> Chars<'a, E> {
	pub fn as_text(&self) -> &'a TextSlice<E> {
		unsafe { TextSlice::new_unchecked(self.1.as_str()) }
	}
}

impl<E> Iterator for Chars<'_, E> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		self.1.next()
	}
}

/// The maximum length of a [`Text`]/[`TextSlice`] when `check_container_length` is enabled.
pub const MAX_LEN: usize = i32::MAX as usize;

#[derive(Debug, PartialEq, Eq)]
pub enum NewTextError {
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	/// Indicates a Knight string was too long.
	LengthTooLong(usize),

	/// Indicates a character within a Knight string wasn't valid.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	IllegalChar {
		/// The char that was invalid.
		chr: char,

		/// The index of the invalid char in the given string.
		index: usize,
	},
}

impl std::fmt::Display for NewTextError {
	fn fmt(&self, #[allow(unused)] f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			#[cfg(feature = "compliance")]
			Self::LengthTooLong(len) => {
				write!(f, "length {len} longer than max {MAX_LEN}")
			}
			#[cfg(feature = "compliance")]
			Self::IllegalChar { chr, index } => {
				write!(f, "illegal char {chr:?} found at index {index}")
			}
		}
	}
}

impl std::error::Error for NewTextError {}

fn validate_len(data: &str, flags: &Flags) -> Result<(), NewTextError> {
	#[cfg(feature = "compliance")]
	if flags.compliance.check_container_length && MAX_LEN < data.len() {
		return Err(NewTextError::LengthTooLong(data.len()));
	}

	let _ = (data, flags);

	Ok(())
}

fn validate(data: &str, flags: &Flags) -> Result<(), NewTextError> {
	#[cfg(feature = "compliance")]
	{
		validate_len(data, flags)?;

		if flags.compliance.knight_encoding {
			for (index, chr) in data.chars().enumerate() {
				if !is_valid_character(chr) {
					// Since everything's a byte, the byte index is the same as the char index.
					return Err(NewTextError::IllegalChar { chr, index });
				}
			}
		}
	}

	let _ = (data, flags);
	Ok(())
}
