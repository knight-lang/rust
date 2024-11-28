use crate::options::Options;
use std::fmt::{self, Debug, Display, Formatter};

/// StringSlice represents a slice of a Knight string, akin to
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct StringSlice<'a>(&'a str);

/// The error that can arise when [creating new StringSlice](StringSlice::new)s.
///
/// Note that unless `compliance` is enabled, this will never be returned.
#[derive(Error, Debug)]
pub enum StringError {
	/// Indicates a Knight string was too long.
	///
	/// This is only ever returned if [`check_container_length`](
	/// crate::env::flags::Compliance::check_container_length) is enabled.
	#[cfg(feature = "compliance")]
	#[error("string is too large ({0} < {len})", len = StringSlice::MAXIMUM_LENGTH)]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	LengthTooLong(usize),

	/// Indicates a character within a string wasn't [valid](is_valid_character).
	///
	/// This is only ever returned if [`knight_encoding`](
	/// crate::env::flags::Compliance::knight_encoding) is enabled.
	#[cfg(feature = "compliance")]
	#[error("{0}")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	EncodingError(#[from] super::EncodingError),
}

impl<'a> StringSlice<'a> {
	/// The maximum length a string can be when compliance checking is enabled.
	#[cfg(feature = "compliance")]
	pub const MAXIMUM_LENGTH: usize = i32::MAX as usize;

	/// Returns a new [`StringSlice`] without doing any forms of validation.
	///
	/// This should only be done for strings which were previously validated, or which are always
	/// valid regardless of the string that's used.
	#[inline]
	pub const fn new_unvalidated(source: &'a str) -> Self {
		Self(source)
	}

	/// Creates a new [`StringSlice`] for the given options. Note that unless the `compliance`
	/// feature is enabled, this function will never fail.
	#[cfg_attr(not(feature = "compliance"), inline)] // inline when we don't have compliance checks.
	pub fn new(source: &'a str, opts: &Options) -> Result<Self, StringError> {
		#[cfg(feature = "compliance")]
		{
			if opts.check_length && Self::MAXIMUM_LENGTH < source.len() {
				return Err(StringError::LengthTooLong(source.len()));
			}

			opts.encoding.validate(source)?;
		}

		Ok(Self(source))
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl Display for StringSlice<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl Debug for StringSlice<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}
