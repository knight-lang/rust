use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::borrow::{Cow, Borrow};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;

mod builder;
mod text_static;
mod cache;

pub use builder::TextBuilder;
pub use text_static::TextStatic;

use std::mem::{align_of, size_of};

#[derive(Debug)]
pub struct InvalidByte {
	pub byte: u8,
	pub pos: usize
}

impl Display for InvalidByte {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let _ = f;
		todo!()
	}
}

impl std::error::Error for InvalidByte {}

const fn validate(data: &[u8]) -> Result<(), InvalidByte> {
	let _ = data;
	Ok(()) // todo
}

// lol no derives
pub struct Text(&'static TextInner);

bitflags::bitflags! {
	#[derive(Default)]
	struct Flags : u8 {
		const NONE         = 0b00000000;
		const STRUCT_ALLOC = 0b00000001;
		const EMBEDDED     = 0b00000010;
		const CACHED       = 0b00000100;
	}
}

#[repr(C)]
struct TextInner {
	refcount: AtomicUsize,
	flags: Flags,
	kind: TextKind
}

#[repr(C)]
union TextKind {
	embed: TextKindEmbedded,
	heap: TextKindPointer,
}

const EMBEDDED_LEN: usize = 64 - size_of::<AtomicUsize>() - size_of::<Flags>() - size_of::<u8>();

sa::const_assert!(EMBEDDED_LEN <= u8::MAX as usize);

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct TextKindEmbedded {
	len: u8,
	data: [u8; EMBEDDED_LEN]
}

const TEXT_KIND_POINTER_PADDING_LEN: usize = align_of::<usize>() - align_of::<Flags>();

sa::const_assert_eq!(
	(align_of::<AtomicUsize>() + align_of::<Flags>() + TEXT_KIND_POINTER_PADDING_LEN) % align_of::<usize>(),
	0
);

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct TextKindPointer {
	_padding: [u8; TEXT_KIND_POINTER_PADDING_LEN],
	len: usize,
	ptr: *const u8
}

sa::assert_eq_align!(Flags, u8);
sa::assert_eq_align!(usize, AtomicUsize);
sa::assert_eq_align!(TextKindPointer, TextKindEmbedded);

unsafe impl Send for TextInner {}
unsafe impl Sync for TextInner {}

unsafe impl Send for Text {}
unsafe impl Sync for Text {}

sa::assert_impl_all!(Text: Send, Sync);
sa::assert_impl_all!(Text: Send, Sync);


impl Clone for Text {
	fn clone(&self) -> Self {
		self.0.refcount.fetch_add(1, SeqCst);

		Self(self.0)
	}
}

impl Drop for Text {
	fn drop(&mut self) {
		if self.0.refcount.fetch_sub(1, SeqCst) != 1 {
			return; // ie we weren't at the end
		}

		if !self.0.flags.contains(Flags::STRUCT_ALLOC) {
			return;
		}

		if !self.is_embedded() {
			unsafe {
				drop(Box::from_raw(self.0.kind.heap.ptr as *mut u8))
			}
		}

		unsafe {
			drop(Box::from_raw(self.0 as *const TextInner as *mut TextStatic));
		}
	}
}

impl Default for Text {
	fn default() -> Self {
		static EMPTY: TextStatic = unsafe { TextStatic::new_unchecked(b"") };

		EMPTY.as_text()
	}
}

impl Text {
	pub fn builder(capacity: usize) -> TextBuilder {
		TextBuilder::with_capacity(capacity)
	}

	pub fn new(bytes: Cow<[u8]>) -> Result<Self, InvalidByte> {
		if let Some(text) = Self::fetch_cached(bytes.borrow()) {
			Ok(text)
		} else {
			Self::new_owned(bytes.into_owned().into_boxed_slice())
		}
	}

	pub fn new_owned(bytes: Box<[u8]>) -> Result<Self, InvalidByte> {
		validate(&bytes)?;

		Ok(Self(Box::leak(Box::new(TextInner {
			refcount: AtomicUsize::new(1),
			flags: Flags::STRUCT_ALLOC,
			kind: TextKind {
				heap: TextKindPointer {
					_padding: [0u8; TEXT_KIND_POINTER_PADDING_LEN],
					len: bytes.len(),
					ptr: Box::into_raw(bytes) as *mut u8
				}
			}
		}))))
	}

	pub fn new_borrowed(bytes: &[u8]) -> Result<Self, InvalidByte> {
		if let Some(text) = Self::fetch_cached(bytes) {
			Ok(text)
		} else {
			Self::new_owned(bytes.to_owned().into_boxed_slice())
		}
	}

	pub fn fetch_cached(bytes: &[u8]) -> Option<Self> {
		Some(cache::fetch_or_insert(bytes, |bytes| bytes.to_owned().into_boxed_slice())) // todo
	}

	const fn is_embedded(&self) -> bool {
		self.0.flags.contains(Flags::EMBEDDED)
	}

	pub fn len(&self) -> usize {
		if self.is_embedded() {
			unsafe { self.0.kind.embed.len as usize }
		} else {
			unsafe { self.0.kind.heap.len }
		}
	}

	pub fn as_ptr(&self) -> *const u8 {
		if self.is_embedded() {
			unsafe { &self.0.kind.embed.data as *const u8 }
		} else {
			unsafe { self.0.kind.heap.ptr }
		}
	}

	pub fn as_bytes(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(self.as_ptr(), self.len())
		}
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(self.as_bytes())
		}
	}
}


impl TryFrom<&str> for Text {
	type Error = InvalidByte;

	fn try_from(str: &str) -> Result<Self, Self::Error> {
		Self::try_from(str.as_bytes())
	}
}

impl TryFrom<&[u8]> for Text {
	type Error = InvalidByte;

	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		Self::new(Cow::Borrowed(bytes))
	}
}

impl TryFrom<String> for Text {
	type Error = InvalidByte;

	fn try_from(string: String) -> Result<Self, InvalidByte> {
		let _ = string;
		// Self::new_owned(string.into_boxed_str().into_boxed_bytes())
		todo!()
	}
}

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Text").field(&self.as_str()).finish()
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.as_str(), f)
	}
}

impl AsRef<str> for Text {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for Text {
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl Eq for Text {}
impl PartialEq for Text {
	fn eq(&self, rhs: &Self) -> bool {
		self == rhs.as_bytes()
	}
}

impl PartialEq<str> for Text {
	fn eq(&self, rhs: &str) -> bool {
		self == rhs.as_bytes()
	}
}

impl PartialEq<[u8]> for Text {
	fn eq(&self, rhs: &[u8]) -> bool {
		self.as_bytes() == rhs
	}
}

impl Hash for Text {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.as_bytes().hash(h)
	}
}

impl PartialOrd for Text {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}

impl PartialOrd<str> for Text {
	fn partial_cmp(&self, rhs: &str) -> Option<std::cmp::Ordering> {
		self.as_bytes().partial_cmp(rhs.as_bytes())
	}
}

impl PartialOrd<[u8]> for Text {
	fn partial_cmp(&self, rhs: &[u8]) -> Option<std::cmp::Ordering> {
		self.as_bytes().partial_cmp(rhs)
	}
}

impl Ord for Text {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_bytes().cmp(rhs.as_bytes())
	}
}

impl Borrow<[u8]> for Text {
	fn borrow(&self) -> &[u8] {
		self.as_bytes()
	}
}

