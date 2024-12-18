use crate::gc::{Flags, Gc, Mark, Sweep};
use std::alloc::Layout;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem::{align_of, size_of, transmute};
use std::sync::atomic::{AtomicU8, Ordering};

use super::{ValueAlign, ValueRepr, ALLOC_VALUE_SIZE_IN_BYTES};
use crate::strings::KnStr;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct KnString(*const Inner);

static EMPTY_INNER: Inner = Inner {
	_alignment: ValueAlign,
	flags: AtomicU8::new(Flags::IsString as u8 | Flags::GcStatic as u8),
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

const ALLOCATED_FLAG: u8 = Flags::IsList as u8; // since we only ever check for islist once
const SIZE_MASK_FLAG: u8 = (Flags::Custom1 as u8) | (Flags::Custom1 as u8) | (Flags::Custom3 as u8);
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

	pub fn into_raw(self) -> ValueRepr {
		unsafe { transmute::<Self, *const Inner>(self) as ValueRepr }
	}

	pub unsafe fn from_raw(raw: ValueRepr) -> Self {
		unsafe { transmute::<*const Inner, Self>(raw as *const Inner) }
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
		unsafe { gc.alloc_value_inner(Flags::IsString as u8 | flags).cast::<Inner>() }
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

unsafe impl Mark for KnString {
	unsafe fn mark(&mut self) {
		self.flags_ref().fetch_or(Flags::GcMarked as u8, Ordering::SeqCst);
	}
}

unsafe impl Sweep for KnString {
	unsafe fn sweep(self, gc: &mut Gc) {
		let old = self.flags_ref().fetch_and(!(Flags::GcMarked as u8), Ordering::SeqCst);

		if old & Flags::GcMarked as u8 == 0 {
			unsafe {
				self.deallocate(gc);
			}
		}
	}

	unsafe fn deallocate(self, gc: &mut Gc) {
		let (flags, inner) = self.flags_and_inner();
		if flags & ALLOCATED_FLAG != 0 {
			unsafe {
				let layout = Layout::from_size_align_unchecked(
					(&raw const (*inner).kind.alloc.len).read(),
					align_of::<u8>(),
				);

				std::alloc::dealloc((&raw mut (*inner).kind.alloc.ptr).read() as *mut u8, layout);
			}
		}

		gc.free_value_inner((self.0 as *mut Inner).cast());
	}
}
