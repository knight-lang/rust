mod builder;
mod character;
mod text;
mod textslice;

pub trait ToText {
	fn to_text(&self, env: &mut Environment) -> crate::Result<Text>;
}

use crate::env::{Environment, Flags};
pub use builder::Builder;
pub use character::Character;
pub use text::*;
pub use textslice::*;

/// Returns whether `chr` is a valid character.
///
/// If [`knight_encoding`](crate::env::flags::Compliance::knight_encoding) is enabled, this will
/// return whether `chr` is a character explicitly allowed by the knight specs. If it's not enabled,
/// `true` will always be returned.
#[inline]
pub const fn is_valid_character(chr: char, flags: &Flags) -> bool {
	#[cfg(feature = "compliance")]
	if flags.compliance.knight_encoding {
		return matches!(chr, '\r' | '\n' | '\t' | ' '..='~');
	}

	true
}

pub struct Chars<'a>(std::str::Chars<'a>);
impl<'a> Chars<'a> {
	pub fn as_text(&self) -> &'a TextSlice {
		unsafe { TextSlice::new_unchecked(self.0.as_str()) }
	}
}

impl Iterator for Chars<'_> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

/// The maximum length of a [`Text`]/[`TextSlice`] when `check_container_length` is enabled.
pub const MAX_LEN: usize = i32::MAX as usize;

/// Problems that can occur when [creating `Text`](Text::new)s.
///
/// The two variants are only ever returned when `compliance` is enabled. But, to keep a uniform
/// type signature for [`Text::new`] regardless of the feature that's enabled, it'll always return
/// a `NewTextError`. However, the variants will only ever be populated if `compliance` is enabled.
#[derive(Debug, PartialEq, Eq)]
pub enum NewTextError {
	/// Indicates a Knight string was too long.
	///
	/// This is only ever returned if [`check_container_length`](
	/// crate::env::flags::Compliance::check_container_length) is enabled.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	LengthTooLong(usize),

	/// Indicates a character within a string wasn't [valid](is_valid_character).
	///
	/// This is only ever returned if [`knight_encoding`](
	/// crate::env::flags::Compliance::knight_encoding) is enabled.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	IllegalChar {
		/// The char that was invalid.
		chr: char,

		/// The index of the invalid char in the given string.
		idx: usize,
	},
}

impl std::fmt::Display for NewTextError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let _ = f;

		#[cfg(feature = "compliance")]
		match *self {
			Self::LengthTooLong(len) => write!(f, "length {len} longer than max {MAX_LEN}"),
			Self::IllegalChar { chr, idx } => write!(f, "illegal char {chr:?} found at index {idx}"),
		}
	}
}

impl std::error::Error for NewTextError {}

const fn validate_len(data: &str, flags: &Flags) -> Result<(), NewTextError> {
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
			for (idx, chr) in data.chars().enumerate() {
				if !is_valid_character(chr, flags) {
					// Since everything's a byte, the byte index is the same as the char index.
					return Err(NewTextError::IllegalChar { chr, idx });
				}
			}
		}
	}

	let _ = (data, flags);
	Ok(())
}
