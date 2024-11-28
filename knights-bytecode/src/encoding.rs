use crate::stringslice::StringError;
use std::fmt::{self, Display, Formatter};

/// Encoding is the different types of encoding this knight implementation supports.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
	/// All UTF-8 strings are valid, i.e. any `str` is a valid Knight string.
	#[cfg_attr(not(feature = "compliance"), default)]
	Utf8,

	/// Only the strict Knight subset is valid.
	#[cfg_attr(feature = "compliance", default)]
	Knight,

	/// Only ASCII-based strings are valid; any other UTF-8 string is invalid.
	Ascii,
}

/// The error that's returned from [`Encoding::validate`].
#[derive(Debug)]
pub struct EncodingError {
	encoding: Encoding,
	position: usize,
	charcater: char,
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
	/// Validate checks to see if `source` only contains valid bytes within the encoding.
	///
	/// Note that this doesn't check for the length of the `source`, which is also required by Knight
	/// compliance, as that's done within [`StringSlice`](crate::StringSlice).
	pub fn validate(self, source: &str) -> Result<(), EncodingError> {
		match self {
			// all `str`s are valid utf8
			Self::Utf8 => Ok(()),

			// Ascii and Knight have to check
			Self::Ascii | Self::Knight => {
				for (idx, chr) in source.bytes().enumerate() {
					let is_invalid = if self == Self::Ascii {
						!chr.is_ascii()
					} else {
						!matches!(chr, b'\r' | b'\n' | b'\t' | b' '..=b'~')
					};

					if is_invalid {
						return Err(EncodingError {
							encoding: self,
							position: idx,
							charcater: chr as char,
						});
					}
				}

				Ok(())
			}
		}
	}
}
