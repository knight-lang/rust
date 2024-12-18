use crate::container::RefCount;
use crate::gc::{self, GarbageCollected, Gc, ValueInner};
use std::alloc::Layout;
use std::fmt::{self, Debug, Formatter};
use std::mem::{align_of, size_of, transmute};
use std::sync::atomic::AtomicU8;

use super::{Value, ValueAlign, ALLOC_VALUE_SIZE_IN_BYTES};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct List(*const Inner);

/// Represents the ability to be converted to a [`List`].
pub trait ToList {
	/// Converts `self` to a [`List`].
	fn to_list(&self, env: &mut crate::Environment) -> crate::Result<List>;
}

static EMPTY_INNER: Inner = Inner {
	_alignment: ValueAlign,
	flags: AtomicU8::new(gc::FLAG_GC_STATIC | gc::FLAG_IS_LIST),
	kind: Kind { embedded: [Value::NULL; MAX_EMBEDDED_LENGTH] },
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
const SIZE_MASK_FLAG: u8 = gc::FLAG_CUSTOM_2 | gc::FLAG_CUSTOM_3;
const SIZE_MASK_SHIFT: u8 = 6;
const MAX_EMBEDDED_LENGTH: usize = (SIZE_MASK_FLAG >> SIZE_MASK_SHIFT) as usize;

// TODO: If this isn't true, we're wasting space!
sa::const_assert!(
	MAX_EMBEDDED_LENGTH == (ALLOC_VALUE_SIZE_IN_BYTES - size_of::<u8>()) / size_of::<Value>()
);

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
	pub const EMPTY: Self = Self(&EMPTY_INNER);

	pub fn into_raw(self) -> *const ValueInner {
		self.0.cast()
	}

	pub unsafe fn from_raw(ptr: *const ValueInner) -> Self {
		Self(ptr.cast())
	}

	pub unsafe fn from_value_inner(raw: *const crate::gc::ValueInner) -> Self {
		Self(raw.cast::<Inner>())
	}

	pub fn boxed(value: Value, gc: &mut Gc) -> Self {
		Self::new(&[value], gc)
	}

	pub fn new(source: &[Value], gc: &mut Gc) -> Self {
		match source.len() {
			0 => Self::default(),
			1..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(source, gc) },
			_ => Self::new_alloc(source, gc),
		}
	}

	fn allocate(flags: u8, gc: &mut Gc) -> *mut Inner {
		unsafe { gc.alloc_value_inner(flags | gc::FLAG_IS_LIST) }.cast::<Inner>()
	}

	fn new_embedded(source: &[Value], gc: &mut Gc) -> Self {
		debug_assert!(source.len() <= MAX_EMBEDDED_LENGTH);
		let inner = Self::allocate((source.len() as u8) << SIZE_MASK_SHIFT, gc);

		unsafe {
			(&raw mut (*inner).kind.embedded)
				.cast::<Value>()
				.copy_from_nonoverlapping(source.as_ptr(), source.len());
		}

		Self(inner)
	}

	fn new_alloc(source: &[Value], gc: &mut Gc) -> Self {
		debug_assert!(source.len() > MAX_EMBEDDED_LENGTH);

		let inner = Self::allocate(ALLOCATED_FLAG, gc);

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

	#[deprecated] // won't work with non-slice types
	fn as_slice(&self) -> &[Value] {
		let (flags, inner) = self.flags_and_inner();

		unsafe {
			let slice_ptr = if flags & ALLOCATED_FLAG != 0 {
				(&raw const (*inner).kind.alloc.ptr).read()
			} else {
				(*inner).kind.embedded.as_ptr()
			};

			std::slice::from_raw_parts(slice_ptr, self.len())
		}
	}

	pub fn len(&self) -> usize {
		let (flags, inner) = self.flags_and_inner();

		if flags & ALLOCATED_FLAG != 0 {
			unsafe { (&raw const (*inner).kind.alloc.len).read() }
		} else {
			(flags as usize) >> SIZE_MASK_SHIFT
		}
	}
}

impl Default for List {
	#[inline]
	fn default() -> Self {
		Self::EMPTY
	}
}

impl Debug for List {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.as_slice(), f)
	}
}

// impl Allocated for KnString {
// }

unsafe impl GarbageCollected for List {
	unsafe fn mark(&self) {
		for value in self.as_slice() {
			unsafe {
				value.mark();
			}
		}
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
				align_of::<Value>(),
			);

			std::alloc::dealloc((&raw mut (*inner).kind.alloc.ptr).read() as *mut u8, layout);
		}
	}
}
