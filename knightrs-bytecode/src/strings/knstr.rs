use crate::options::Options;
use crate::value::{Integer, KnValueString, List, ToList};
use crate::{Environment, Value};
use std::fmt::{self, Debug, Display, Formatter};

/// KnStr represents a slice of a Knight string, akin to rust's `str`
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KnStr(str);

/// The error that can arise when [creating new KnStr](KnStr::new)s.
///
/// Note that unless `compliance` is enabled, this will never be returned.
#[derive(Error, Debug)]
pub enum StringError {
	/// Indicates a Knight string was too long.
	///
	/// This is only ever returned if [`check_container_length`](
	/// crate::env::flags::Compliance::check_container_length) is enabled.
	#[cfg(feature = "compliance")]
	#[error("string is too large ({0} < {len})", len = KnStr::COMPLIANCE_MAX_LEN)]
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

impl KnStr {
	/// The maximum length a string can be when compliance checking is enabled.
	pub const COMPLIANCE_MAX_LEN: usize = i32::MAX as usize;

	/// Creates a new [`KnStr`] without doing any forms of validation.
	///
	/// # Compliance
	/// The `source` that's passed in should be a valid Knight string under all compliance features.
	/// More specifically, that means that its length must never be more than [`COMPLIANCE_MAX_LEN`],
	/// and that [`Encoding::Knight::validate`] should pass for it.
	///
	/// [`COMPLIANCE_MAX_LEN`]: Self::COMPLIANCE_MAX_LEN
	/// [`Encoding::Knight::validate`]: super::Encoding::Knight::validate
	#[inline]
	pub const fn new_unvalidated(source: &str) -> &Self {
		#[cfg(feature = "compliance")] // Only enable debug checks in compliance mode
		{
			debug_assert!(source.len() <= Self::COMPLIANCE_MAX_LEN);
			debug_assert!(super::Encoding::Knight.validate(source).is_ok());
		}

		// SAFETY: `KnStr`s are `#[repr(transparent)]` around `str`s
		unsafe { &*(source as *const str as *const Self) }
	}

	/// Creates a new [`KnStr`] without doing any forms of validation.
	///
	/// # Errors
	/// If the `compliance` option is disabled, this function never fails.
	///
	/// If `opts.compliance.check_container_length` is enabled, and `source.len()` is greater than
	/// [`COMPLIANCE_MAX_LEN`](Self::COMPLIANCE_MAX_LEN), an [`StringError::LengthTooLong`] is
	/// returned.
	///
	/// The `opts.encoding` also validates the source.
	#[cfg_attr(not(feature = "compliance"), inline)] // inline when we don't have compliance checks.
	pub fn new<'a>(source: &'a str, opts: &Options) -> Result<&'a Self, StringError> {
		// TODO: Combine with new_validate_length ?

		#[cfg(feature = "compliance")]
		{
			if opts.compliance.check_container_length && Self::COMPLIANCE_MAX_LEN < source.len() {
				return Err(StringError::LengthTooLong(source.len()));
			}

			opts.encoding.validate(source)?;
		}

		// SAFETY: `KnStr`s are `#[repr(transparent)]` around `str`s
		Ok(unsafe { &*(source as *const str as *const Self) })
	}

	pub fn into_boxed(&self) -> Box<Self> {
		let rawbox = Box::into_raw(self.as_str().to_string().into_boxed_str());

		// SAFETY: same layout
		unsafe { Box::from_raw(rawbox as *mut Self) }
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn repeat(&self, amount: usize, opts: &Options) -> Result<KnValueString, StringError> {
		// Make sure `str.repeat()` won't panic
		if amount.checked_mul(self.len()).map_or(true, |c| isize::MAX as usize <= c) {
			// TODO: maybe we don't have the length in `LengthTooLong` ?
			// return Err(StringError::LengthTooLong(self.len().wrapping_mul(amount)));
			todo!();
		}

		#[cfg(feature = "compliance")]
		if opts.compliance.check_container_length && Self::COMPLIANCE_MAX_LEN < self.len() * amount {
			return Err(StringError::LengthTooLong(self.len() * amount));
		}

		Ok(KnValueString::from_string_unchecked(self.as_str().repeat(amount)))
	}

	/// Gets an iterate over [`Character`]s.
	pub fn chars(&self) -> std::str::Chars<'_> {
		self.0.chars()
	}

	pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
		let substring = self.0.get(range)?;

		// SAFETY: We're getting a substring of a valid TextSlice, which thus will itself be valid.
		Some(Self::new_unvalidated(substring))
	}

	/// Concatenates two strings together
	pub fn concat(&self, rhs: &Self, opts: &Options) -> Result<KnValueString, StringError> {
		panic!();
		// let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

		todo!()
		// builder.push(self);
		// builder.push(rhs);

		// builder.finish(flags)
	}

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn split(&self, sep: &Self, env: &mut Environment) -> List {
		if sep.is_empty() {
			// TODO: optimize me
			return Value::from(self.to_owned()).to_list(env).unwrap();
		}

		// SAFETY: If `self` is within the container bounds, so is the length of its chars.
		List::new_unvalidated(
			self
				.as_str()
				.split(sep.as_str())
				.map(|s| KnValueString::new_unvalidated(s.to_string()))
				.map(Value::from),
		)
	}

	pub fn ord(&self, opts: &Options) -> crate::Result<Integer> {
		let chr = self.chars().next().ok_or(crate::Error::DomainError("empty string"))?;
		// technically not redundant in case checking for ints is enabled but not strings.
		Integer::new_error(u32::from(chr) as _, opts).map_err(From::from)
	}

	/// Gets the first character of `self`, if it exists.
	pub fn head(&self) -> Option<char> {
		self.chars().next()
	}

	/// Gets everything _but_ the first character of `self`, if it exists.
	pub fn tail(&self) -> Option<&Self> {
		self.get(1..)
	}

	pub fn remove_substr(&self, substr: &Self) -> KnValueString {
		let _ = substr;
		todo!();
	}
}

impl Display for KnStr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl Debug for KnStr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl Default for &'_ KnStr {
	fn default() -> Self {
		KnStr::new_unvalidated("")
	}
}

impl ToOwned for KnStr {
	type Owned = crate::value::KnValueString;

	fn to_owned(&self) -> crate::value::KnValueString {
		crate::value::KnValueString::from_slice(self)
	}
}

impl KnStr {
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
	// pub fn concat(&self, rhs: &Self, opts: &Options) -> Result<KnValueString, NewTextError> {
	// 	let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

	// 	builder.push(self);
	// 	builder.push(rhs);

	// 	builder.finish(flags)
	// }

	// pub fn repeat(&self, amount: usize, opts: &Options) -> Result<KnValueString, NewTextError> {
	// 	unsafe { KnValueString::new_len_unchecked((**self).repeat(amount), flags) }
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
	// 		.map(|x| unsafe { KnValueString::new_unchecked(x) }.into())
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

	// pub fn remove_substr(&self, substr: &Self) -> KnValueString {
	// 	let _ = substr;
	// 	todo!();
	// }
}
