use crate::{value::Number, Error};
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KnStr(str);

impl Default for &KnStr {
	fn default() -> Self {
		KnStr::new("").unwrap()
	}
}

impl Display for KnStr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl Deref for KnStr {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for KnStr {
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
	// All valid `str`s are valid KnStr is normal mode.
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

impl KnStr {
	pub const fn new(inp: &str) -> Result<&Self, IllegalChar> {
		if let Err(err) = validate(inp) {
			return Err(err);
		}

		#[allow(unsafe_code)]
		// SAFETY: Since `KnStr` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Ok(unsafe { &*(inp as *const str as *const Self) })
	}

	pub fn to_boxed(&self) -> Box<Self> {
		self.0.to_string().into_boxed_str().try_into().unwrap()
	}

	pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
		Some(Self::new(self.0.get(range)?).unwrap())
	}
}

impl<'a> TryFrom<&'a str> for &'a KnStr {
	type Error = IllegalChar;

	#[inline]
	fn try_from(inp: &'a str) -> Result<Self, Self::Error> {
		KnStr::new(inp)
	}
}

impl TryFrom<Box<str>> for Box<KnStr> {
	type Error = IllegalChar;

	fn try_from(inp: Box<str>) -> Result<Self, Self::Error> {
		validate(&inp)?;

		#[allow(unsafe_code)]
		// SAFETY: Since `KnStr` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		Ok(unsafe { Box::from_raw(Box::into_raw(inp) as _) })
	}
}

impl<'a> From<&'a KnStr> for &'a str {
	#[inline]
	fn from(kstr: &'a KnStr) -> Self {
		&kstr
	}
}

impl From<Box<KnStr>> for Box<str> {
	#[inline]
	fn from(kstr: Box<KnStr>) -> Self {
		// SAFETY: Since `KnStr` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		unsafe { Box::from_raw(Box::into_raw(kstr) as _) }
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharedStr(Rc<KnStr>);

impl Default for SharedStr {
	fn default() -> Self {
		SharedStr::new("").unwrap()
	}
}

impl Display for SharedStr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl std::ops::Deref for SharedStr {
	type Target = KnStr;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<Box<KnStr>> for SharedStr {
	fn from(kstr: Box<KnStr>) -> Self {
		Self(kstr.into())
	}
}
impl TryFrom<String> for SharedStr {
	type Error = IllegalChar;

	fn try_from(inp: String) -> Result<Self, Self::Error> {
		let boxed = Box::<KnStr>::try_from(inp.into_boxed_str())?;

		Ok(Self(boxed.into()))
	}
}

impl SharedStr {
	pub fn new(inp: impl ToString) -> Result<Self, IllegalChar> {
		inp.to_string().try_into()
	}

	pub fn to_number(&self) -> crate::Result<Number> {
		todo!()
		// let mut chars = text.trim().bytes();
		// let mut sign = 1;
		// let mut number: Number = 0;

		// match chars.next() {
		// 	Some(b'-') => sign = -1,
		// 	Some(b'+') => { /* do nothing */ }
		// 	Some(digit @ b'0'..=b'9') => number = (digit - b'0') as _,
		// 	_ => return Ok(0),
		// };

		// while let Some(digit @ b'0'..=b'9') = chars.next() {
		// 	cfg_if! {
		// 		 if #[cfg(feature="checked-overflow")] {
		// 			  number = number
		// 					.checked_mul(10)
		// 					.and_then(|num| num.checked_add((digit as u8 - b'0') as _))
		// 					.ok_or_else(|| {
		// 						 let err: Error = error_inplace!(Error::KnStringConversionOverflow);
		// 						 err
		// 					})?;
		// 		 } else {
		// 			  number = number.wrapping_mul(10).wrapping_add((digit as u8 - b'0') as _);
		// 		 }
		// 	}
		// }

		// Ok(sign * number) // todo: check for this erroring. ?
	}
}
