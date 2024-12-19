use crate::container::RefCount;
use crate::gc::{self, GarbageCollected, Gc, ValueInner};
use std::alloc::Layout;
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem::{align_of, size_of, transmute};
use std::sync::atomic::AtomicU8;

use super::{Value, ValueAlign, ALLOC_VALUE_SIZE_IN_BYTES};

#[repr(transparent)]
pub struct List<'gc>(*const Inner<'gc>);

pub(crate) mod consts {
	use super::*;

	pub const JUST_TRUE: List = List(&JUST_TRUE_INNER);
	static JUST_TRUE_INNER: Inner = Inner {
		_alignment: ValueAlign,
		// TODO: make the `FLAG_CUSTOM_2` use a function.
		flags: AtomicU8::new(
			gc::FLAG_GC_STATIC | ALLOCATED_FLAG | gc::FLAG_IS_LIST | gc::FLAG_CUSTOM_2,
		),
		kind: Kind { embedded: [Value::TRUE; MAX_EMBEDDED_LENGTH] },
	};
}

/// Represents the ability to be converted to a [`List`].
pub trait ToList<'gc> {
	/// Converts `self` to a [`List`].
	fn to_list(&self, env: &mut crate::Environment) -> crate::Result<List<'gc>>;
}

#[repr(C)]
struct Inner<'gc> {
	_alignment: ValueAlign,
	flags: AtomicU8,
	kind: Kind<'gc>,
}

sa::assert_eq_align!(crate::gc::ValueInner, Inner);
sa::assert_eq_size!(crate::gc::ValueInner, Inner);

// SAFETY: We never deallocate it without flags, and flags are atomicu8. TODO: actual gc
unsafe impl Send for Inner<'_> {}

// SAFETY: We never deallocate it without flags, and flags are atomicu8. TODO: actual gc
unsafe impl Sync for Inner<'_> {}

const ALLOCATED_FLAG: u8 = gc::FLAG_CUSTOM_0;
const SIZE_MASK_FLAG: u8 = gc::FLAG_CUSTOM_2 | gc::FLAG_CUSTOM_3;
const SIZE_MASK_SHIFT: u8 = 6;
const MAX_EMBEDDED_LENGTH: usize = (SIZE_MASK_FLAG >> SIZE_MASK_SHIFT) as usize;

// TODO: If this isn't true, we're wasting space!
sa::const_assert!(
	MAX_EMBEDDED_LENGTH == (ALLOC_VALUE_SIZE_IN_BYTES - size_of::<u8>()) / size_of::<Value>()
);

#[repr(C)]
union Kind<'gc> {
	embedded: [Value<'gc>; MAX_EMBEDDED_LENGTH],
	alloc: Alloc<'gc>,
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
struct Alloc<'gc> {
	_padding: [u8; ALLOC_PADDING_ALIGN],
	ptr: *const Value<'gc>,
	len: usize,
}

sa::const_assert_eq!(size_of::<Inner<'_>>(), ALLOC_VALUE_SIZE_IN_BYTES);
sa::assert_eq_size!(List, super::Value);

impl Default for List<'_> {
	#[inline]
	fn default() -> Self {
		static EMPTY_INNER: Inner<'_> = Inner {
			_alignment: ValueAlign,
			flags: AtomicU8::new(gc::FLAG_GC_STATIC | gc::FLAG_IS_LIST),
			kind: Kind { embedded: [Value::NULL; MAX_EMBEDDED_LENGTH] },
		};
		Self(&EMPTY_INNER)
	}
}
impl<'gc> List<'gc> {
	pub fn into_raw(self) -> *const ValueInner {
		self.0.cast()
	}

	pub unsafe fn from_raw(ptr: *const ValueInner) -> Self {
		Self(ptr.cast())
	}

	pub fn boxed(value: Value<'gc>, gc: &'gc Gc) -> Self {
		Self::new(&[value], gc)
	}

	pub fn new(source: &[Value<'gc>], gc: &'gc Gc) -> Self {
		match source.len() {
			0 => Self::default(),
			1..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(source, gc) },
			_ => Self::new_alloc(source, gc),
		}
	}

	fn allocate(flags: u8, gc: &'gc Gc) -> *mut Inner<'gc> {
		unsafe { gc.alloc_value_inner(flags | gc::FLAG_IS_LIST) }.cast::<Inner>()
	}

	fn new_embedded(source: &[Value<'gc>], gc: &'gc Gc) -> Self {
		debug_assert!(source.len() <= MAX_EMBEDDED_LENGTH);
		let inner = Self::allocate((source.len() as u8) << SIZE_MASK_SHIFT, gc);

		unsafe {
			(&raw mut (*inner).kind.embedded)
				.cast::<Value<'gc>>()
				.copy_from_nonoverlapping(source.as_ptr(), source.len());
		}

		Self(inner)
	}

	fn new_alloc(source: &[Value<'gc>], gc: &'gc Gc) -> Self {
		debug_assert!(source.len() > MAX_EMBEDDED_LENGTH);

		let inner = Self::allocate(ALLOCATED_FLAG, gc);

		unsafe {
			(&raw mut (*inner).kind.alloc.len).write(source.len());

			let ptr = std::alloc::alloc(Layout::from_size_align_unchecked(
				source.len(),
				align_of::<Value<'_>>(),
			))
			.cast::<Value<'_>>();
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

impl Debug for List<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.as_slice(), f)
	}
}

// impl Allocated for KnString {
// }

unsafe impl GarbageCollected for List<'_> {
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
