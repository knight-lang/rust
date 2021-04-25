// use std::collections::HashSet;
// use once_cell::sync::OnceCell;

use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::mem::{size_of, ManuallyDrop};
use std::fmt::{self, Debug, Display, Formatter};
use std::convert::TryFrom;
use std::borrow::Borrow;

bitflags::bitflags! {
	#[repr(C)]
	#[derive(Default)]
	struct Flags: u8 {
		const STRUCT_ALLOC = 0b00000001;
		const EMBEDDED     = 0b00000010;
		const STATIC       = 0b00000100;
		const CACHED       = 0b00001000;
	}
}

#[repr(transparent)]
pub struct Text(&'static TextInner);

#[repr(C)]
#[doc(hidden)] // needs to be pub for `static_text`
pub struct TextInner {
	refcount: AtomicUsize,
	flags: Flags,
	kind: TextKind,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StaticText(&'static TextInner);

unsafe impl Send for StaticText {}
unsafe impl Sync for StaticText {}

impl TextInner {
	#[doc(hidden)]
	pub const fn new_const(data: &'static str) -> Self {
		Self {
			refcount: AtomicUsize::new(0),
			flags: Flags::STATIC,
			kind: TextKind {
				heap: TextHeap {
					len: data.len(),
					ptr: data.as_ptr()
				}
			}
		}
	}

	#[inline(always)]
	pub const fn into_text(&'static self) -> StaticText {
		StaticText(self)
	}
}

impl StaticText {
	#[inline(always)]
	pub const fn text(self) -> Text {
		Text(self.0)
	}
}

#[repr(C)] // todo: align with flags to make it better.
union TextKind {
	embed: TextEmbed,
	heap: TextHeap,
	r#static: &'static str
}

const EMBEDDED_LEN: usize = 32 - (size_of::<Flags>() + size_of::<usize>() + size_of::<u8>());

sa::const_assert!(EMBEDDED_LEN <= u8::MAX as usize);

#[repr(C)]
#[derive(Clone, Copy)]
struct TextEmbed {
	len: u8,
	data: [u8; EMBEDDED_LEN]
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TextHeap {
	len: usize,
	ptr: *const u8
}

unsafe impl Send for Text {}
unsafe impl Sync for Text {}

impl Text {
	// const fn new_const(data: &'static str) -> Self {
	// 	let refcount = AtomicUsize::new(0);
	// 	let flags = Flags::STATIC;
	// 	let kind = TextKind { r#static: data };

	// 	Self(&TextInner { refcount, flags, kind })
	// }

	fn _alloc(flags: Flags, kind: TextKind) -> Self {
		Self(Box::leak(Box::new(TextInner {
			refcount: AtomicUsize::new(1),
			flags: flags | Flags::STRUCT_ALLOC,
			kind
		})))
	}

	/// Creates a new [`Text`] with the given data.
	///
	/// [`InvalidByte`] is returned if `data` is not a valid Knight string.
	pub fn new<S: ToString + AsRef<str>>(data: S) -> Result<Self, InvalidByte> {
		let data = data.to_string();

		validate(data.as_bytes())?;

		Ok(unsafe { Self::new_unchecked(data) })
	}

	/// Creates a new [`Text`] without verifying that `data` is valid.
	pub unsafe fn new_unchecked<S: ToString + AsRef<str>>(data: S) -> Self {
		debug_assert!(validate(data.as_ref().as_bytes()).is_ok());

		let mut data = ManuallyDrop::new(data.to_string());

		Self::_alloc(Flags::STRUCT_ALLOC, TextKind {
			heap: TextHeap {
				len: data.len(),
				ptr: data.as_mut_ptr()
			}
		})
	}

	/// Whether or not this `self` has an embedded string.
	fn is_embedded(&self) -> bool {
		self.0.flags.contains(Flags::EMBEDDED)
	}

	/// Whether or not `self` was heap allocated.
	fn is_struct_alloc(&self) -> bool {
		self.0.flags.contains(Flags::STRUCT_ALLOC)
	}

	/// How many references to `self` exist.
	pub fn refcount(&self) -> usize {
		self.0.refcount.load(SeqCst)
	}

	/// Gets `self`'s data as a slice.
	pub fn as_slice(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(self.as_ptr(), self.len())
		}
	}

	/// Gets `self`'s data as a `str`.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(self.as_slice())
		}
	}

	/// A pointer to `self`'s data.
	///
	/// Note: This should not be cast to a mutable pointer unless you have
	/// unique ownership of this struct (ie [`refcount()`] is one).
	pub fn as_ptr(&self) -> *const u8 {
		unsafe {
			if self.is_embedded() {
				&self.0.kind.embed.data as *const u8
			} else {
				self.0.kind.heap.ptr
			}
		}
	}

	/// Fetches the length of `self`.
	///
	/// This is both the length in bytes and chars, as [`Text`]s only contain a subset of ASCII.
	pub fn len(&self) -> usize {
		unsafe {
			if self.is_embedded() {
				self.0.kind.embed.len as usize
			} else {
				self.0.kind.heap.len
			}
		}
	}

	/// Completely deallocate `self`. 
	unsafe fn deallocate(&mut self) {
		debug_assert_eq!(self.0.refcount.load(SeqCst), 0, "deallocating a live struct?");

		// free allocated data
		if !self.is_embedded() {
			drop(Box::from_raw(self.0.kind.heap.ptr as *mut u8));
		}

		if self.is_struct_alloc() {
			drop(Box::from_raw(self.0 as *const TextInner as *mut TextInner));
		}
	}
}

impl Clone for Text {
	fn clone(&self) -> Self {
		self.0.refcount.fetch_add(1, SeqCst);

		Self(self.0)
	}
}

impl Drop for Text {
	fn drop(&mut self) {
		if self.is_struct_alloc() && self.0.refcount.fetch_sub(1, SeqCst) == 0 {
			unsafe {
				self.deallocate();
			}
		}
	}
}

impl Default for Text {
	fn default() -> Self {
		static EMPTY: StaticText = static_text!("");
		
		EMPTY.text()
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

impl AsRef<str> for Text {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for Text {
	fn as_ref(&self) -> &[u8] {
		self.as_slice()
	}
}

impl Eq for Text {}
impl PartialEq for Text {
	fn eq(&self, rhs: &Self) -> bool {
		self.as_slice() == rhs.as_slice()
	}
}

impl PartialEq<str> for Text {
	fn eq(&self, rhs: &str) -> bool {
		self.as_str() == rhs
	}
}

impl PartialEq<[u8]> for Text {
	fn eq(&self, rhs: &[u8]) -> bool {
		self.as_slice() == rhs
	}
}

impl PartialOrd for Text {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}

impl Ord for Text {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_slice().cmp(rhs.as_slice())
	}
}

impl Borrow<str> for Text {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl Borrow<[u8]> for Text {
	fn borrow(&self) -> &[u8] {
		self.as_slice()
	}
}

impl std::ops::Deref for Text {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

/// An error that indicates a character within a Knight string wasn't valid.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidByte {
	/// The byte that was invalid.
	pub byte: u8,

	/// The index of the invalid byte in the given string.
	pub idx: usize
}

impl Display for InvalidByte {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "invalid byte {:?} found at position {}", self.byte, self.idx)
	}
}

impl std::error::Error for InvalidByte {}

/// Checks to see if `byte` is a valid knight character.
#[must_use]
pub const fn is_byte_valid(byte: u8) -> bool {
	matches!(byte, b'\r' | b'\n' | b'\t' | b' '..=b'~')
}

fn validate(data: &[u8]) -> Result<(), InvalidByte> {
	for (idx, &byte) in data.iter().enumerate() {
		if !is_byte_valid(byte) {
			return Err(InvalidByte { byte, idx });
		}
	}

	Ok(())
}

impl TryFrom<&str> for Text {
	type Error = InvalidByte;

	#[inline]
	fn try_from(string: &str) -> Result<Self, Self::Error> {
		Self::new(string)
	}
}

impl TryFrom<String> for Text {
	type Error = InvalidByte;

	#[inline]
	fn try_from(string: String) -> Result<Self, Self::Error> {
		Self::new(string)
	}
}

/// A Builder for [`Text`]s.
pub struct TextBuilder(Inner);

enum Inner {
	Embedded(u8, [u8; EMBEDDED_LEN]),
	Heap(Box<[u8]>)
}

impl TextBuilder {
	pub fn with_capacity(capacity: usize) -> Self {
		if capacity <= EMBEDDED_LEN {
			Self(Inner::Embedded(capacity as u8, [0; EMBEDDED_LEN]))
		} else {
			Self(Inner::Heap(vec![0; capacity].into_boxed_slice()))
		}
	}

	pub fn build(self) -> Result<Text, InvalidByte> {
		if matches!(self.0, Inner::Embedded(0, _)) {
			return Ok(Text::default());
		}

		validate(&self.as_ref())?;

		Ok(match self.0 {
			Inner::Embedded(len, data) =>
				Text::_alloc(Flags::EMBEDDED, TextKind {
					embed: TextEmbed { len, data }
				}),
			Inner::Heap(heap) => 
				Text::_alloc(Flags::default(), TextKind {
					heap: TextHeap {
						len: heap.len(),
						ptr: Box::into_raw(heap) as *const _ as *mut u8
					}
				})
		})
	}
}

impl AsRef<[u8]> for TextBuilder {
	fn as_ref(&self) -> &[u8] {
		match self.0 {
			Inner::Embedded(len, ref data) => &data[..len as usize],
			Inner::Heap(ref slice) => slice
		}
	}
}

impl AsMut<[u8]> for TextBuilder {
	fn as_mut(&mut self) -> &mut [u8] {
		match self.0 {
			Inner::Embedded(len, ref mut data) => &mut data[..len as usize],
			Inner::Heap(ref mut slice) => slice
		}
	}
}
