use crate::options::Options;

#[repr(transparent)]
pub struct StringSlice(str);

pub const MAX_STRING_LENGTH: usize = i32::MAX as usize;

#[derive(Debug)]
pub enum StringError {
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

impl StringSlice {
	pub fn new<'a>(source: &'a str, opts: &Options) -> Result<&'a Self, StringError> {
		#[cfg(feature = "compliance")]
		{
			if opts.check_length && source.len() > MAX_STRING_LENGTH {
				return Err(StringError::LengthTooLong(source.len()));
			}

			opts.encoding.validate(source)?;
		}

		// SAFETY: `StringSlice` has the same layout as `str`.
		Ok(unsafe { &*(source as *const str as *const Self) })
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}
