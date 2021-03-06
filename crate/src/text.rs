//! Types relating to the [`Text`].

use std::sync::Arc;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use std::borrow::Borrow;

cfg_if! {
	if #[cfg(feature = "unsafe-single-threaded")] {
		use once_cell::unsync::OnceCell;
	} else {
		use once_cell::sync::OnceCell;
	}
}

cfg_if! {
	if #[cfg(not(feature = "cache-strings"))] {
		// do nothing
	} else if #[cfg(feature = "unsafe-single-threaded")] {
		use std::collections::HashSet;
		static mut TEXT_CACHE: OnceCell<HashSet<Text>> = OnceCell::new();

	} else {
		use std::collections::HashSet;
		use std::sync::RwLock;

		static TEXT_CACHE: OnceCell<RwLock<HashSet<Text>>> = OnceCell::new();
	}
}

/// The string type within Knight.
#[derive(Clone)]
pub struct Text(
	#[cfg(all(not(feature="cache-strings"), not(feature="unsafe-single-threaded")))]
	std::sync::Arc<str>,

	#[cfg(all(not(feature="cache-strings"), feature="unsafe-single-threaded"))]
	std::rc::Rc<str>,

	#[cfg(feature="cache-strings")]
	&'static str,
);

impl Default for Text {
	#[allow(unsafe_code)]
	fn default() -> Self {
		// we need it mut so we can have !Send/!Sync in a static.
		static mut  EMPTY: OnceCell<Text> = OnceCell::new();

		unsafe { &EMPTY }.get_or_init(|| unsafe { Self::new_unchecked("") }).clone()
	}
}

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(self.as_str(), f)
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(self.as_str(), f)
	}
}

impl Hash for Text {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.as_str().hash(h)
	}
}

impl Borrow<str> for Text {
	fn borrow(&self) -> &str {
		self.as_ref()
	}
}

impl Eq for Text {}
impl PartialEq for Text {
	fn eq(&self, rhs: &Self) -> bool {
		self.as_str() == rhs.as_str()
	}
}

impl PartialOrd for Text {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		self.as_str().partial_cmp(rhs.as_str())
	}
}

impl Ord for Text {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_str().cmp(rhs.as_str())
	}
}

/// An error that indicates a character within a Knight string wasn't valid.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidChar {
	/// The byte that was invalid.
	pub chr: char,

	/// The index of the invalid byte in the given string.
	pub idx: usize
}

impl Display for InvalidChar {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "invalid byte {:?} found at position {}", self.chr, self.idx)
	}
}

impl std::error::Error for InvalidChar {}

/// Checks to see if `chr` is a valid knight character.
#[must_use]
pub const fn is_valid_char(chr: char) -> bool {
	return !cfg!(feature="disallow-unicode") || matches!(chr, '\r' | '\n' | '\t' | ' '..='~');
}

fn validate_string(data: &str) -> Result<(), InvalidChar> {
	for (idx, chr) in data.chars().enumerate() {
		if !is_valid_char(chr) {
			return Err(InvalidChar { chr, idx });
		}
	}

	Ok(())
}

impl Text {
	/// Creates a new `Text` with the given input string.
	///
	/// # Errors
	/// If `string` contains any characters which aren't valid in Knight source code, an `InvalidChar` is returned.
	///
	/// # See Also
	/// - [`Text::new_unchecked`] For a version which doesn't verify `string`.
	#[must_use = "Creating an Text does nothing on its own"]
	pub fn new<T: ToString + Borrow<str> + ?Sized>(string: &T) -> Result<Self, InvalidChar> {
		validate_string(string.borrow())?;

		#[allow(unsafe_code)] // we just verified it.
		Ok(unsafe { Self::new_unchecked(string) })
	}

	/// Creates a new `Text`, without verifying that the string is valid.
	///
	/// # Safety
	/// All characters within the string must be valid for Knight strings. See the specs for what exactly this entails.
	#[must_use = "Creating an Text does nothing on its own"]
	#[allow(unsafe_code)]
	pub unsafe fn new_unchecked<T: ToString + Borrow<str> + ?Sized>(string: &T) -> Self {
		debug_assert_eq!(validate_string(string.borrow()), Ok(()), "invalid string encountered: {:?}", string.borrow());

		#[cfg(not(feature="cache-strings"))] {
			Self(string.to_string().into())
		}

		#[cfg(feature="cache-strings")] {
			// initialize if it's not been initialized yet.
			let cache = TEXT_CACHE.get_or_init(Default::default);

			// in the unsafe-single-threaded version, we simply use the one below.
			#[cfg(not(feature="unsafe-single-threaded"))]
			if let Some(text) = cache.read().unwrap().get(string.borrow()) {
				return text.clone();
			}

			drop(cache);

			let mut cache = {
				#[cfg(feature="unsafe-single-threaded")]
				{ TEXT_CACHE.get_mut().unwrap_or_else(|| unreachable!()) }

				#[cfg(not(feature="unsafe-single-threaded"))]
				{ TEXT_CACHE.get().unwrap().write().unwrap() }
			};

			if let Some(text) = cache.get(string.borrow()) {
				text.clone()
			} else {
				let leaked = Box::leak(string.to_string().into_boxed_str());
				cache.insert(Text(leaked));
				Text(leaked)
			}
		}
	}

	/// Gets a reference to the contained string.
	#[inline]
	#[must_use]
	pub fn as_str(&self) -> &str {
		self.0.as_ref()
	}
}

impl TryFrom<&str> for Text {
	type Error = InvalidChar;

	#[inline]
	fn try_from(string: &str) -> Result<Self, Self::Error> {
		Self::new(string)
	}
}

impl TryFrom<String> for Text {
	type Error = InvalidChar;

	#[inline]
	fn try_from(string: String) -> Result<Self, Self::Error> {
		Self::new(&string)
	}
}

impl AsRef<str> for Text {
	#[inline]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl std::ops::Deref for Text {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}
