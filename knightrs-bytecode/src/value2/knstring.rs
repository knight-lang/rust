use std::alloc::Layout;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use super::ALLOC_VALUE_SIZE_IN_BYTES;
use crate::container::RefCount;
use crate::strings::KnStr;

#[repr(transparent)]
pub struct KnString(Option<NonNull<Inner>>);

#[repr(C, packed)]
struct Inner {
	flags: u8,
	kind: Kind,
}

#[repr(u8, align(1))]
enum Flags {
	// If unset, it's embedded
	Allocated = 0b0000_0001,
	GcMarked = 0b0000_0010,
	SizeMask = 0b1111_1000,
}

const FLAG_SIZE_SHIFT: usize = 3;
sa::const_assert!((Flags::SizeMask as usize) >> FLAG_SIZE_SHIFT <= MAX_EMBEDDED_LENGTH);
const MAX_EMBEDDED_LENGTH: usize = ALLOC_VALUE_SIZE_IN_BYTES - std::mem::size_of::<u8>();

#[repr(C, packed)]
union Kind {
	embeded: [u8; MAX_EMBEDDED_LENGTH],
	alloc: Alloc,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Alloc {
	_blank: [u8; std::mem::align_of::<*const u8>() - 1],
	ptr: *const (),
	len: usize,
	refcount: usize,
}

// const x: [u8; ALLOC_VALUE_SIZE_IN_BYTES] = [0; ]
sa::const_assert_eq!(std::mem::size_of::<Inner>(), ALLOC_VALUE_SIZE_IN_BYTES);

impl Default for KnString {
	#[inline]
	fn default() -> Self {
		Self(None)
	}
}

impl KnString {
	pub fn into_raw(self) -> usize {
		unsafe { std::mem::transmute::<Option<NonNull<Inner>>, usize>(self.0) }
	}

	pub unsafe fn from_raw(raw: usize) -> Self {
		Self(unsafe { std::mem::transmute::<usize, Option<NonNull<Inner>>>(raw) })
	}

	pub fn new(source: &KnStr) -> Self {
		match source.len() {
			0 => Self(None),
			1..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(source) },
			_ => Self::new_alloc(source),
		}
	}

	fn new_embedded(source: &KnStr) -> Self {
		unsafe {
			// TODO: use custom heap allocator
			let inner =
				std::ptr::NonNull::new(std::alloc::alloc(Layout::new::<Inner>()).cast::<Inner>())
					.expect("alloc failed");

			debug_assert!(source.len() <= MAX_EMBEDDED_LENGTH);

			(&raw mut (*inner.as_ptr()).flags).write((source.len() as u8) << FLAG_SIZE_SHIFT);
			(&raw mut (*inner.as_ptr()).kind.embeded)
				.cast::<u8>()
				.copy_from_nonoverlapping(source.as_str().as_ptr(), source.len());

			Self(Some(inner))
		}
	}

	fn inner(&self) -> Option<&Inner> {
		self.0.map(|inner| unsafe { inner.as_ref() })
	}

	fn new_alloc(source: &KnStr) -> Self {
		todo!()
	}

	pub fn as_knstr(&self) -> &KnStr {
		&self
	}

	pub fn len(&self) -> usize {
		let Some(inner) = self.inner() else {
			return 0;
		};

		if inner.flags & Flags::Allocated as u8 == 1 {
			todo!();
		} else {
			(inner.flags as usize) >> FLAG_SIZE_SHIFT
		}
	}
}

impl std::ops::Deref for KnString {
	type Target = KnStr;

	fn deref(&self) -> &Self::Target {
		let Some(inner) = self.inner() else {
			return KnStr::EMPTY;
		};

		if inner.flags & Flags::Allocated as u8 == 1 {
			todo!("deref allocated");
		}

		unsafe {
			let slice = std::slice::from_raw_parts(inner.kind.embeded.as_ptr(), self.len());
			KnStr::new_unvalidated(std::str::from_utf8_unchecked(slice))
		}
	}
}

impl Debug for KnString {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.as_knstr(), f)
	}
}

impl Display for KnString {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.as_knstr(), f)
	}
}
