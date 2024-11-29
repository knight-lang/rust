use crate::options::Options;
use std::fmt::{self, Debug, Display, Formatter};

/// StringSlice represents a slice of a Knight string, akin to
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct StringSlice(str);

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

impl StringSlice {
	/// The maximum length a string can be when compliance checking is enabled.
	#[cfg(feature = "compliance")]
	pub const MAXIMUM_LENGTH: usize = i32::MAX as usize;

	/// Returns a new [`StringSlice`] without doing any forms of validation.
	///
	/// This should only be done for strings which were previously validated, or which are always
	/// valid regardless of the string that's used.
	#[inline]
	pub const fn new_unvalidated(source: &str) -> &Self {
		// SAFETY: layout is the same
		unsafe { &*(source as *const str as *const Self) }
	}

	/// Creates a new [`StringSlice`] for the given options. Note that unless the `compliance`
	/// feature is enabled, this function will never fail.
	#[cfg_attr(not(feature = "compliance"), inline)] // inline when we don't have compliance checks.
	pub fn new<'a>(source: impl AsRef<str>, opts: &Options) -> Result<&'a Self, StringError> {
		let source = source.as_ref();

		#[cfg(feature = "compliance")]
		{
			if opts.compliance.check_length && Self::MAXIMUM_LENGTH < source.len() {
				return Err(StringError::LengthTooLong(source.len()));
			}

			opts.encoding.validate(source)?;
		}

		// SAFETY:
		Ok(unsafe { &*(source as *const str as *const Self) })
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl Display for StringSlice {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl Debug for StringSlice {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl Default for &'_ StringSlice {
	fn default() -> Self {
		StringSlice::new_unvalidated("")
	}
}

impl ToOwned for StringSlice {
	type Owned = crate::value::KString;

	fn to_owned(&self) -> crate::value::KString {
		crate::value::KString::from_slice(self)
	}
}

impl StringSlice {
	// /// Gets an iterate over [`Character`]s.
	// pub fn chars(&self) -> Chars<'_> {
	// 	Chars(self.0.chars())
	// }

	// pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
	// 	let substring = self.0.get(range)?;

	// 	// SAFETY: We're getting a substring of a valid TextSlice, which thus will itself be valid.
	// 	Some(unsafe { Self::new_unchecked(substring) })
	// }

	// /// Concatenates two strings together
	// pub fn concat(&self, rhs: &Self, flags: &Flags) -> Result<Text, NewTextError> {
	// 	let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

	// 	builder.push(self);
	// 	builder.push(rhs);

	// 	builder.finish(flags)
	// }

	// pub fn repeat(&self, amount: usize, flags: &Flags) -> Result<Text, NewTextError> {
	// 	unsafe { Text::new_len_unchecked((**self).repeat(amount), flags) }
	// }

	// #[cfg(feature = "extensions")]
	// #[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	// pub fn split(&self, sep: &Self, env: &mut Environment) -> List {
	// 	if sep.is_empty() {
	// 		// TODO: optimize me
	// 		return Value::from(self.to_owned()).to_list(env).unwrap();
	// 	}

	// 	let chars = (**self)
	// 		.split(&**sep)
	// 		.map(|x| unsafe { Text::new_unchecked(x) }.into())
	// 		.collect::<Vec<_>>();

	// 	// SAFETY: If `self` is within the container bounds, so is the length of its chars.
	// 	unsafe { List::new_unchecked(chars) }
	// }

	// pub fn ord(&self) -> crate::Result<Integer> {
	// 	Integer::try_from(self.chars().next().ok_or(crate::Error::DomainError("empty string"))?)
	// }

	// /// Gets the first character of `self`, if it exists.
	// pub fn head(&self) -> Option<char> {
	// 	self.chars().next()
	// }

	// /// Gets everything _but_ the first character of `self`, if it exists.
	// pub fn tail(&self) -> Option<&TextSlice> {
	// 	self.get(1..)
	// }

	// pub fn remove_substr(&self, substr: &Self) -> Text {
	// 	let _ = substr;
	// 	todo!();
	// }
}
