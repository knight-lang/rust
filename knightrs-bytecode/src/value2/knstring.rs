use crate::gc::{self, GarbageCollected, Gc, ValueInner};
use std::alloc::Layout;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem::{align_of, size_of, transmute};
use std::sync::atomic::{AtomicU8, Ordering};

use super::{ValueAlign, ALLOC_VALUE_SIZE_IN_BYTES};
use crate::strings::KnStr;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct KnString(*const Inner);

/// Represents the ability to be converted to a [`KnString`].
pub trait ToKnString {
	/// Converts `self` to a [`KnString`].
	fn to_knstring(&self, env: &mut crate::Environment) -> crate::Result<KnString>;
}

pub(crate) mod consts {
	use super::*;

	macro_rules! static_str {
		($id:literal) => {{
			static __INNER: Inner = Inner {
				_alignment: ValueAlign,
				// TODO: make the `FLAG_CUSTOM_2` use a function.
				flags: AtomicU8::new(gc::FLAG_GC_STATIC | gc::FLAG_IS_LIST | ALLOCATED_FLAG),
				kind: Kind {
					alloc: Alloc {
						_padding: [0; ALLOC_PADDING_ALIGN],
						ptr: $id.as_ptr(),
						len: $id.len(),
					},
				},
			};
			KnString(&__INNER)
		}};
	}

	pub const TRUE: KnString = static_str!("true");
	pub const FALSE: KnString = static_str!("false");
}

static EMPTY_INNER: Inner = Inner {
	_alignment: ValueAlign,
	flags: AtomicU8::new(gc::FLAG_IS_STRING | gc::FLAG_GC_STATIC),
	kind: Kind { embedded: [0; MAX_EMBEDDED_LENGTH] },
};

#[repr(C)]
struct Inner {
	_alignment: ValueAlign,
	flags: AtomicU8,
	kind: Kind,
}

sa::assert_eq_align!(crate::gc::ValueInner, Inner);
sa::assert_eq_size!(crate::gc::ValueInner, Inner);

// SAFETY: We never deallocate it without flags, and flags are atomicu8. TODO: actual gc
unsafe impl Send for Inner {}

// SAFETY: We never deallocate it without flags, and flags are atomicu8. TODO: actual gc
unsafe impl Sync for Inner {}

const ALLOCATED_FLAG: u8 = gc::FLAG_CUSTOM_0;
const SIZE_MASK_FLAG: u8 = gc::FLAG_CUSTOM_1 | gc::FLAG_CUSTOM_2 | gc::FLAG_CUSTOM_3;
const SIZE_MASK_SHIFT: u8 = 5;
const MAX_EMBEDDED_LENGTH: usize = (SIZE_MASK_FLAG >> SIZE_MASK_SHIFT) as usize;

#[repr(C)]
union Kind {
	embedded: [u8; MAX_EMBEDDED_LENGTH],
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
	ptr: *const u8,
	len: usize,
}

sa::const_assert_eq!(size_of::<Inner>(), ALLOC_VALUE_SIZE_IN_BYTES);
sa::assert_eq_size!(KnString, super::Value);

impl Default for KnString {
	#[inline]
	fn default() -> Self {
		Self::EMPTY
	}
}

impl KnString {
	pub const EMPTY: Self = Self(&EMPTY_INNER);

	pub fn into_raw(self) -> *const ValueInner {
		self.0.cast()
	}

	pub unsafe fn from_raw(raw: *const ValueInner) -> Self {
		Self(raw.cast())
	}

	pub unsafe fn from_value_inner(raw: *const crate::gc::ValueInner) -> Self {
		Self(raw.cast::<Inner>())
	}

	pub fn new(source: &KnStr, gc: &mut Gc) -> Self {
		match source.len() {
			0 => Self::default(),
			1..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(source, gc) },
			_ => Self::new_alloc(source, gc),
		}
	}

	fn allocate(flags: u8, gc: &mut Gc) -> *mut Inner {
		unsafe { gc.alloc_value_inner(gc::FLAG_IS_STRING as u8 | flags).cast::<Inner>() }
	}

	fn new_embedded(source: &KnStr, gc: &mut Gc) -> Self {
		debug_assert!(source.len() <= MAX_EMBEDDED_LENGTH);
		let inner = Self::allocate((source.len() as u8) << SIZE_MASK_SHIFT, gc);

		unsafe {
			(&raw mut (*inner).kind.embedded)
				.cast::<u8>()
				.copy_from_nonoverlapping(source.as_str().as_ptr(), source.len());
		}

		Self(inner)
	}

	fn new_alloc(source: &KnStr, gc: &mut Gc) -> Self {
		debug_assert!(source.len() > MAX_EMBEDDED_LENGTH);

		let inner = Self::allocate(ALLOCATED_FLAG, gc);

		unsafe {
			(&raw mut (*inner).kind.alloc.len).write(source.len());

			let ptr =
				std::alloc::alloc(Layout::from_size_align_unchecked(source.len(), align_of::<u8>()));
			if ptr.is_null() {
				panic!("alloc failed");
			}

			ptr.copy_from_nonoverlapping(source.as_str().as_ptr(), source.len());
			(&raw mut (*inner).kind.alloc.ptr).write(ptr);
		}

		Self(inner)
	}

	fn flags_ref(&self) -> &AtomicU8 {
		unsafe { &*&raw const (*self.0).flags }
	}

	fn flags_and_inner(&self) -> (u8, *mut Inner) {
		unsafe {
			// TODO: orderings
			((*&raw const (*self.0).flags).load(Ordering::SeqCst), self.0 as _)
		}
	}

	pub fn as_knstr(&self) -> &KnStr {
		&self
	}

	pub fn len(self) -> usize {
		let (flags, inner) = self.flags_and_inner();

		if flags & ALLOCATED_FLAG as u8 != 0 {
			unsafe { (&raw const (*inner).kind.alloc.len).read() }
		} else {
			(flags as usize) >> SIZE_MASK_SHIFT
		}
	}
}

impl std::ops::Deref for KnString {
	type Target = KnStr;

	fn deref(&self) -> &Self::Target {
		let (flags, inner) = self.flags_and_inner();

		unsafe {
			let slice_ptr = if flags & ALLOCATED_FLAG != 0 {
				(&raw const (*inner).kind.alloc.ptr).read()
			} else {
				(*inner).kind.embedded.as_ptr()
			};

			let slice = std::slice::from_raw_parts(slice_ptr, self.len());
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

unsafe impl GarbageCollected for KnString {
	unsafe fn mark(&self) {
		// Do nothing, `self` doesn't reference other `GarbageCollected `types.
		// TODO: If we add in "cons" variants and whatnot, then this should be modified
	}

	unsafe fn deallocate(self) {
		let (flags, inner) = self.flags_and_inner();
		debug_assert_eq!(flags & gc::FLAG_GC_STATIC, 0, "<called deallocate on a static?>");

		// If the string isn't allocated, then just return early.
		if flags & ALLOCATED_FLAG == 0 {
			return;
		}

		// Free the memory associated with the allocated pointer.
		unsafe {
			let layout = Layout::from_size_align_unchecked(
				(&raw const (*inner).kind.alloc.len).read(),
				align_of::<u8>(),
			);

			std::alloc::dealloc((&raw mut (*inner).kind.alloc.ptr).read() as *mut u8, layout);
		}
	}
}
