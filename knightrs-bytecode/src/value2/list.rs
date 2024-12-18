use crate::container::RefCount;
use std::alloc::Layout;
use std::fmt::{self, Debug, Formatter};
use std::mem::{align_of, size_of, transmute};
use std::sync::atomic::AtomicU8;

use super::{Value, ValueAlign, ValueRepr, ALLOC_VALUE_SIZE_IN_BYTES};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct List(*const Inner);

static EMPTY_INNER: Inner = Inner {
	_alignment: ValueAlign,
	flags: AtomicU8::new(Flags::Static as u8),
	kind: Kind { embedded: [Value::NULL; MAX_EMBEDDED_LENGTH] },
};

#[repr(C)]
struct Inner {
	_alignment: ValueAlign,
	flags: AtomicU8,
	kind: Kind,
}

// SAFETY: We never deallocate it without flags, and flags are atomicu8. TODO: actual gc
unsafe impl Send for Inner {}

// SAFETY: We never deallocate it without flags, and flags are atomicu8. TODO: actual gc
unsafe impl Sync for Inner {}

#[repr(u8, align(1))]
enum Flags {
	// If unset, it's embedded
	Allocated = 0b0000_0001,
	GcMarked = 0b0000_0010,
	Static = 0b0000_0100,
	SizeMask = 0b1100_0000,
}

const MAX_EMBEDDED_LENGTH: usize =
	(ALLOC_VALUE_SIZE_IN_BYTES - size_of::<u8>()) / size_of::<Value>();
const FLAG_SIZE_SHIFT: usize = 6;
sa::const_assert!((Flags::SizeMask as usize) >> FLAG_SIZE_SHIFT <= MAX_EMBEDDED_LENGTH);

#[repr(C)]
union Kind {
	embedded: [Value; MAX_EMBEDDED_LENGTH],
	alloc: Alloc,
}

const ALLOC_PADDING_ALIGN: usize =
	(if align_of::<*const u8>() >= align_of::<usize>() {
		align_of::<*const u8>()
	} else {
		align_of::<usize>()
	}) - align_of::<u8>() // minus align of flags
;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Alloc {
	_padding: [u8; ALLOC_PADDING_ALIGN],
	ptr: *const Value,
	len: usize,
}

sa::const_assert_eq!(size_of::<Inner>(), ALLOC_VALUE_SIZE_IN_BYTES);
sa::assert_eq_size!(List, super::Value);

impl List {
	pub fn into_raw(self) -> ValueRepr {
		unsafe { transmute::<Self, *const Inner>(self) as ValueRepr }
	}

	pub unsafe fn from_raw(raw: ValueRepr) -> Self {
		unsafe { transmute::<*const Inner, Self>(raw as *const Inner) }
	}

	pub fn new(source: &[Value]) -> Self {
		match source.len() {
			0..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(source) },
			_ => Self::new_alloc(source),
		}
	}

	fn allocate(flags: u8) -> *mut Inner {
		unsafe {
			let inner = std::alloc::alloc(Layout::new::<Inner>()).cast::<Inner>();
			if inner.is_null() {
				panic!("alloc failed");
			}

			(&raw mut (*inner).flags).write(AtomicU8::new(flags));

			inner
		}
	}

	fn new_embedded(source: &[Value]) -> Self {
		debug_assert!(source.len() <= MAX_EMBEDDED_LENGTH);
		let inner = Self::allocate((source.len() as u8) << FLAG_SIZE_SHIFT);

		unsafe {
			(&raw mut (*inner).kind.embedded)
				.cast::<Value>()
				.copy_from_nonoverlapping(source.as_ptr(), source.len());
		}

		Self(inner)
	}

	fn new_alloc(source: &[Value]) -> Self {
		debug_assert!(source.len() > MAX_EMBEDDED_LENGTH);

		let inner = Self::allocate((source.len() as u8) << FLAG_SIZE_SHIFT);

		unsafe {
			(&raw mut (*inner).kind.alloc.len).write(source.len());

			let ptr =
				std::alloc::alloc(Layout::from_size_align_unchecked(source.len(), align_of::<Value>()))
					.cast::<Value>();
			if ptr.is_null() {
				panic!("alloc failed");
			}

			ptr.copy_from_nonoverlapping(source.as_ptr(), source.len());
			(&raw mut (*inner).kind.alloc.ptr).write(ptr);
		}

		Self(inner)
	}

	fn flags_and_inner(&self) -> (u8, *mut Inner) {
		unsafe {
			// TODO: orderings
			((*&raw const (*self.0).flags).load(std::sync::atomic::Ordering::Relaxed), self.0 as _)
		}
	}

	pub fn as_slice(&self) -> &[Value] {
		&self
	}

	pub fn len(&self) -> usize {
		let (flags, inner) = self.flags_and_inner();

		if flags & Flags::Allocated as u8 == 1 {
			unsafe { (&raw const (*inner).kind.alloc.len).read() }
		} else {
			(flags as usize) >> FLAG_SIZE_SHIFT
		}
	}
}

impl std::ops::Deref for List {
	type Target = [Value];

	fn deref(&self) -> &Self::Target {
		let (flags, inner) = self.flags_and_inner();

		unsafe {
			let slice_ptr = if flags & Flags::Allocated as u8 == 1 {
				(&raw const (*inner).kind.alloc.ptr).read()
			} else {
				(*inner).kind.embedded.as_ptr()
			};

			std::slice::from_raw_parts(slice_ptr, self.len())
		}
	}
}

impl Debug for List {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.as_slice(), f)
	}
}
