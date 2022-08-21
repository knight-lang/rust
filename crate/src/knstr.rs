use crate::{Error, Integer};
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KnStr(str);

impl Default for &KnStr {
	#[inline]
	fn default() -> Self {
		// SAFETY: we know that `""` is a valid string, as it contains nothing.
		unsafe { KnStr::new_unchecked("") }
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
	/// Creates a new `KnStr` without validating `inp`.
	///
	/// # Safety
	/// - `inp` must be a valid `KnStr`.
	pub const unsafe fn new_unchecked(inp: &str) -> &Self {
		debug_assert!(validate(inp).is_ok());

		// SAFETY: Since `KnStr` is a `repr(transparent)` wrapper around `str`, we're able to
		// safely transmute.
		&*(inp as *const str as *const Self)
	}

	pub const fn new(inp: &str) -> Result<&Self, IllegalChar> {
		match validate(inp) {
			// SAFETY: we justverified it was valid
			Ok(_) => Ok(unsafe { Self::new_unchecked(inp) }),

			// Can't use `?` or `Result::map` in const functions
			Err(err) => Err(err),
		}
	}

	pub fn chars(&self) -> Chars<'_> {
		Chars(self.0.chars())
	}

	pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
		let substring = self.0.get(range)?;

		// SAFETY: We're getting a substring of a valid KnStr, which thus will itself be valid.
		Some(unsafe { Self::new_unchecked(substring) })
	}

	pub fn concat(&self, rhs: &Self) -> SharedStr {
		let mut cat = String::with_capacity(self.len() + rhs.len());
		cat.push_str(self);
		cat.push_str(rhs);

		SharedStr::try_from(cat).unwrap()
	}

	pub fn repeat(&self, amount: usize) -> SharedStr {
		(**self).repeat(amount).try_into().unwrap()
	}
}

impl<'a> TryFrom<&'a str> for &'a KnStr {
	type Error = IllegalChar;

	#[inline]
	fn try_from(inp: &'a str) -> Result<Self, Self::Error> {
		KnStr::new(inp)
	}
}

impl TryFrom<&str> for SharedStr {
	type Error = IllegalChar;

	#[inline]
	fn try_from(inp: &str) -> Result<Self, Self::Error> {
		<&KnStr>::try_from(inp).map(From::from)
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
		kstr
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
pub struct SharedStr(
	#[cfg(not(feature = "multithreaded"))] std::rc::Rc<KnStr>,
	#[cfg(feature = "multithreaded")] std::sync::Arc<KnStr>,
);

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(SharedStr: Send, Sync);

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

	pub fn to_integer(&self) -> crate::Result<Integer> {
		let mut bytes = self.trim_start().bytes();

		let (is_negative, mut number) = match bytes.next() {
			Some(b'+') => (false, 0),
			Some(b'-') => (true, 0),
			Some(num @ b'0'..=b'9') => (false, (num - b'0') as Integer),
			_ => return Ok(0),
		};

		while let Some(digit @ b'0'..=b'9') = bytes.next() {
			#[cfg(feature = "checked-overflow")]
			{
				number = number
					.checked_mul(10)
					.and_then(|num| num.checked_add((digit as u8 - b'0') as _))
					.ok_or(Error::IntegerOverflow)?;
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				number = (number * 10) + (digit - b'0') as Integer;
			}
		}

		if is_negative {
			#[cfg(feature = "checked-overflow")]
			{
				number = number.checked_neg().ok_or(Error::IntegerOverflow)?;
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				number = -number;
			}
		}

		Ok(number)
	}
}

impl std::borrow::Borrow<KnStr> for SharedStr {
	fn borrow(&self) -> &KnStr {
		self
	}
}

impl From<&KnStr> for SharedStr {
	fn from(knstr: &KnStr) -> Self {
		Box::<KnStr>::try_from(knstr.to_string().into_boxed_str()).unwrap().into()
	}
}

pub struct Chars<'a>(std::str::Chars<'a>);

impl<'a> Chars<'a> {
	pub fn as_knstr(&self) -> &'a KnStr {
		unsafe { KnStr::new_unchecked(self.0.as_str()) }
	}
}

impl Iterator for Chars<'_> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}
