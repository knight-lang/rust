//! Types relating to the [`Text`].
use std::sync::Arc;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use std::ptr::NonNull;
use std::borrow::Borrow;
use std::ops::{Add, Mul, Deref};

use crate::boolean::{ToBoolean, Boolean};
use crate::number::{ToNumber, Number};

#[cfg(all(not(feature="cache-strings"), not(feature="unsafe-single-threaded")))]
use std::sync::atomic::{AtomicUsize, Ordering};

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

mod cow;
mod r#ref;
mod r#static;
pub use cow::TextCow;
pub use r#ref::TextRef;
pub use r#static::TextStatic;

/// The string type within Knight.
pub struct Text(NonNull<Inner>);

#[repr(C, align(8))]
struct Inner {
	len: usize,
	#[cfg(all(not(feature="cache-strings"), not(feature="unsafe-single-threaded")))]
	rc: AtomicUsize,
	#[cfg(all(not(feature="cache-strings"), feature="unsafe-single-threaded"))]
	rc: usize,
	data: [u8; 0]
}

const_assert!(8 <= std::mem::align_of::<Inner>());

pub trait ToText {
	fn to_text(&self) -> crate::Result<TextCow>;
}

impl Clone for Text {
	fn clone(&self) -> Self {
		cfg_if! {
			if #[cfg(feature="cache-strings")] {
				// do nothing
			} else if #[cfg(feature="unsafe-single-threaded")] {
				unsafe { self.0.as_mut().rc += 1; }
			} else {
				unsafe { self.0.as_ref().rc.fetch_add(1, Ordering::SeqCst); }
			}
		}

		Self(self.0)
	}
}

impl Drop for Text {
	fn drop(&mut self) {
		unsafe {
			cfg_if! {
				if #[cfg(feature="cache-strings")] {
					return; // do notihng
				} else if #[cfg(feature="unsafe-single-threaded")] {
					self.0.as_mut().rc -= 1;
					if self.0.as_ref().rc != 0 {
						return;
					}

				} else {
					if self.0.as_ref().rc.fetch_sub(1, Ordering::SeqCst) != 0 {
						return;
					}
				}
			}

			std::ptr::drop_in_place(self.0.as_ptr());
		}
	}
}

impl Default for Text {
	#[cfg(not(feature="cache-strings"))]
	fn default() -> Self {
		// we need it mut so we can have !Send/!Sync in a static.
		static mut EMPTY: OnceCell<Text> = OnceCell::new();

		unsafe { &EMPTY }.get_or_init(|| unsafe { Self::new_unchecked("") }).clone()
	}

	#[cfg(feature="cache-strings")]
	#[inline]
	fn default() -> Self {
		Self(&ThinStr::default())
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
	#[cfg(not(feature="cache-strings"))]
	fn eq(&self, rhs: &Self) -> bool {
		self.as_str() == rhs.as_str()
	}

	#[cfg(feature="cache-strings")]
	#[inline]
	fn eq(&self, rhs: &Self) -> bool {
		self.0 as *const _ == rhs.0 as *const _
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
	if data.len() > isize::MAX as usize {
		// technically this isn't the character at that position, but it'd be extremely expensive to find it.
		return Err(InvalidChar { chr: '\0', idx: isize::MAX as usize + 1 })
	}

	for (idx, chr) in data.chars().enumerate() {
		if !is_valid_char(chr) {
			return Err(InvalidChar { chr, idx });
		}
	}

	Ok(())
}

// safety: len cannot be zero
unsafe fn allocate_inner(string: &str) -> NonNull<Inner> {
	use std::alloc::{Layout, alloc, handle_alloc_error};
	use std::mem::{size_of, align_of};
	use std::ptr::{write, copy_nonoverlapping};

	debug_assert_ne!(string.len(), 0);

	let layout = Layout::from_size_align(
		size_of::<Inner>() + string.len(),
		align_of::<Inner>()
	).unwrap();

	let inner = alloc(layout) as *mut Inner;

	if inner.is_null() {
		handle_alloc_error(layout);
	}

	write(&mut (*inner).len, string.len());
	#[cfg(not(feature="cache-strings"))] { 
		write(&mut (*inner).rc, 1.into());
	}
	copy_nonoverlapping(string.as_ptr(), &mut (*inner).data as *mut _ as *mut u8, string.len());

	NonNull::new_unchecked(inner)
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
	pub fn new(string: &str) -> Result<Self, InvalidChar> {
		validate_string(string).map(|_| unsafe { Self::new_unchecked(string) })
	}

	/// Creates a new `Text`, without verifying that the string is valid.
	///
	/// # Safety
	/// All characters within the string must be valid for Knight strings. See the specs for what exactly this entails.
	#[must_use = "Creating an Text does nothing on its own"]
	pub unsafe fn new_unchecked(string: &str) -> Self {
		debug_assert_eq!(validate_string(string), Ok(()), "invalid string encountered: {:?}", string);
		debug_assert!(string.len() <= isize::MAX as usize);

		if string.len() == 0 {
			return Self::default();
		}

		#[cfg(not(feature="cache-strings"))] {
			Self(unsafe { allocate_inner(string) })
		}

		#[cfg(feature="cache-strings")] {
			if string.is_empty() {
				return Self::default();
			}

			// initialize if it's not been initialized yet.
			let cache = TEXT_CACHE.get_or_init(Default::default);

			// in the unsafe-single-threaded version, we simply use the one below.
			#[cfg(not(feature="unsafe-single-threaded"))]
			if let Some(text) = cache.read().unwrap().get(string) {
				return text.clone();
			}

			drop(cache);

			let mut cache = {
				#[cfg(feature="unsafe-single-threaded")]
				{ TEXT_CACHE.get_mut().unwrap_or_else(|| unreachable!()) }

				#[cfg(not(feature="unsafe-single-threaded"))]
				{ TEXT_CACHE.get().unwrap().write().unwrap() }
			};

			if let Some(text) = cache.get(string) {
				text.clone()
			} else {
				let inner = unsafe { allocate_inner(string) };
				cache.insert(Text(inner));
				Text(inner)
			}
		}
	}

	pub(crate) fn into_raw(self) -> *const () {
		self.0.as_ptr() as *const ()
	}

	// safety: must be a valid pointer returned from `into_raw`
	pub(crate) unsafe fn from_raw(raw: *const ()) -> Self {
		Self(NonNull::new_unchecked(raw as *mut Inner))
	}

	// safety: must be a valid pointer returned from `into_raw`
	pub(crate) unsafe fn str_from_raw<'a>(raw: *const ()) -> &'a str {
		todo!()
	}

	// safety: must be a valid pointer returned from `into_raw`
	#[inline]
	pub(crate) unsafe fn clone_in_place(raw: *const ()) {
		cfg_if! {
			if #[cfg(feature="cache-strings")] {
				// do nothing
			} else if #[cfg(feature="unsafe-single-threaded")] {
				(*(raw as *mut Inner)).rc += 1;
			} else {
				(*(raw as *mut Inner)).rc.fetch_add(1, Ordering::SeqCst);
			}
		}
	}

	// safety: must be a valid pointer returned from `into_raw`
	#[inline]
	pub(crate) unsafe fn drop_in_place(raw: *const ()) {
		drop(Self::from_raw(raw));
	}

	/// Gets a reference to the contained string.
	#[inline]
	#[must_use]
	pub fn as_str(&self) -> &str {
		use std::{str, slice};

		unsafe {
			str::from_utf8_unchecked(slice::from_raw_parts(
				&self.0.as_ref().data as *const _ as *const _,
				self.0.as_ref().len
			))
		}
	}
}

impl TryFrom<&str> for Text {
	type Error = InvalidChar;

	#[inline]
	fn try_from(string: &str) -> Result<Self, InvalidChar> {
		Self::new(string)
	}
}

impl TryFrom<String> for Text {
	type Error = InvalidChar;

	#[inline]
	fn try_from(string: String) -> Result<Self, InvalidChar> {
		Self::new(&string)
	}
}

impl AsRef<str> for Text {
	#[inline]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Deref for Text {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl Add<&Text> for &Text {
	type Output = Text;

	fn add(self, rhs: &Text) -> Self::Output {
		// todo: use caches
		let mut buf = String::with_capacity(self.len() + rhs.len());

		buf.push_str(&self);
		buf.push_str(&rhs);

		// todo: construct a string in-place
		unsafe {
			Text::new_unchecked(&buf)
		}
	}
}

impl Mul<usize> for &Text {
	type Output = Text;

	fn mul(self, rhs: usize) -> Self::Output {
		let mut data = String::with_capacity(self.len() * rhs);

		for _ in 0..rhs {
			data.push_str(&self);
		}

		unsafe {
			Text::new_unchecked(&data)
		}
	}
}

impl ToText for Text {
	fn to_text(&self) -> crate::Result<TextCow<'_>> {
		Ok(TextRef::new(self).into())
	}
}

impl ToBoolean for Text {
	fn to_boolean(&self) -> crate::Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToNumber for Text {
	fn to_number(&self) -> crate::Result<Number> {
		todo!();
	}
}