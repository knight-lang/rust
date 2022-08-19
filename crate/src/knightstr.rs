use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KnightStr(str);

impl Default for &KnightStr {
	fn default() -> Self {
		KnightStr::new("").unwrap()
	}
}

impl Display for KnightStr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl Deref for KnightStr {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for KnightStr {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// An error that indicates a character within a Knight string wasn't valid.
#[derive(Debug, PartialEq, Eq)]
pub struct IllegalChar {
	/// The char that was invalid.
	pub chr: char,

	/// The index of the invalid char in the given string.
	pub index: usize,
}

impl Display for IllegalChar {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "illegal byte {:?} found at position {}", self.chr, self.index)
	}
}

impl std::error::Error for IllegalChar {}

/// Returns whether `chr` is a character that can appear within Knight.
///
/// Normally, every character is considered valid. However, when the `disallow-unicode` feature is
/// enabled, only characters which are explicitly mentioned in the Knight spec are allowed.
#[inline]
pub const fn is_valid(chr: char) -> bool {
	if cfg!(feature = "strict-charset") {
		matches!(chr, '\r' | '\n' | '\t' | ' '..='~')
	} else {
		true
	}
}

const fn validate(data: &str) -> Result<(), IllegalChar> {
	// All valid `str`s are valid KnightStr is normal mode.
	if cfg!(not(feature = "strict-charset")) {
		return Ok(());
	}

	// We're in const context, so we must use `while` with bytes.
	// Since we're not using unicode, everything's just a byte anyways.
	let bytes = data.as_bytes();
	let mut index = 0;

	while index < bytes.len() {
		let chr = bytes[index] as char;

		if !is_valid(chr) {
			// Since everything's a byte, the byte index is the same as the char index.
			return Err(IllegalChar { chr, index });
		}

		index += 1;
	}

	Ok(())
}

impl KnightStr {
	pub const fn new(inp: &str) -> Result<&Self, IllegalChar> {
		if let Err(err) = validate(inp) {
			return Err(err);
		}

		#[allow(unsafe_code)]
		// SAFETY: Since `KnightStr` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Ok(unsafe { &*(inp as *const str as *const Self) })
	}

	pub fn to_boxed(&self) -> Box<Self> {
		self.0.to_string().into_boxed_str().try_into().unwrap()
	}
}

impl<'a> TryFrom<&'a str> for &'a KnightStr {
	type Error = IllegalChar;

	#[inline]
	fn try_from(inp: &'a str) -> Result<Self, Self::Error> {
		KnightStr::new(inp)
	}
}

impl TryFrom<Box<str>> for Box<KnightStr> {
	type Error = IllegalChar;

	fn try_from(inp: Box<str>) -> Result<Self, Self::Error> {
		validate(&inp)?;

		#[allow(unsafe_code)]
		// SAFETY: Since `KnightStr` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Ok(unsafe { Box::from_raw(Box::into_raw(inp) as _) })
	}
}

impl<'a> From<&'a KnightStr> for &'a str {
	#[inline]
	fn from(kstr: &'a KnightStr) -> Self {
		&kstr
	}
}

impl From<Box<KnightStr>> for Box<str> {
	#[inline]
	fn from(kstr: Box<KnightStr>) -> Self {
		// SAFETY: Since `KnightStr` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		unsafe { Box::from_raw(Box::into_raw(kstr) as _) }
	}
}
