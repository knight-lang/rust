use std::fmt::{self, Display, Formatter};

/// Encoding is the different types of encoding this knight implementation supports.
///
/// Note that the `compliance` feature needs to be enabled to use anything other than
/// [`Encoding::Utf8`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
	/// All UTF-8 strings are valid, i.e. any `str` is a valid Knight string.
	#[cfg_attr(not(feature = "compliance"), default)]
	Utf8,

	/// Only the strict Knight subset is valid.
	#[cfg(feature = "compliance")]
	#[cfg_attr(feature = "compliance", default)]
	Knight,

	/// Only ASCII-based strings are valid; any other UTF-8 string is invalid.
	#[cfg(feature = "compliance")]
	Ascii,
}

/// The error that's returned from [`Encoding::validate`].
#[derive(Debug, PartialEq, Eq)]
pub struct EncodingError {
	pub encoding: Encoding, // todo: dont make pub lol make fns
	pub position: usize,
	pub character: char,
}

impl std::error::Error for EncodingError {}
impl Display for EncodingError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(
			f,
			"encoding error: character {:?} at offset {} isn't valid in {:?} encoding",
			self.character, self.position, self.encoding
		)
	}
}

impl Encoding {
	pub const fn is_char_valid(self, chr: char) -> bool {
		match self {
			Self::Utf8 => true,

			#[cfg(feature = "compliance")]
			Self::Ascii => chr.is_ascii(),

			#[cfg(feature = "compliance")]
			Self::Knight => matches!(chr, '\r' | '\n' | '\t' | ' '..='~'),
		}
	}

	/// Validate checks to see if `source` only contains valid bytes within the encoding.
	///
	/// Note that this doesn't check for the length of the `source`, which is also required by Knight
	/// compliance, as that's done within [`KnStr`](crate::KnStr).
	///
	/// This will always return `Ok(())` unless the `compliance` feature is enabled (as the only
	/// encoding is [`Encoding::Utf8`]).
	#[cfg_attr(not(feature = "compliance"), inline)] // inline it when it can never fail.
	pub const fn validate(self, source: &str) -> Result<(), EncodingError> {
		match self {
			// all `str`s are valid utf8
			Self::Utf8 => Ok(()),

			// Ascii and Knight have to check
			#[cfg(feature = "compliance")]
			Self::Ascii | Self::Knight => {
				let mut idx = 0;
				let bytes = source.as_bytes();

				// Gotta do it this way b/c we're in a const function
				while idx < bytes.len() {
					let chr = bytes[idx];

					if !self.is_char_valid(chr as char) {
						return Err(EncodingError {
							encoding: self,
							position: idx,
							character: chr as char,
						});
					}

					idx += 1;
				}

				Ok(())
			}
		}
	}
}
