use crate::gc::{self, GarbageCollected, Gc, ValueInner};
use std::alloc::Layout;
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem::{align_of, size_of, transmute, MaybeUninit};
use std::sync::atomic::{AtomicU8, Ordering};

use super::{ValueAlign, ALLOC_VALUE_SIZE_IN_BYTES};
use crate::strings::KnStr;

/// A KnString represents an allocated string within Knight, and is garbage collected.
///
/// (It's `Kn` because `String` is already a type in Rust, and I didn't want confusion.)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct KnString<'gc>(*const Inner, PhantomData<&'gc ()>);

/// Represents the ability to be converted to a [`KnString`].
pub trait ToKnString<'gc> {
	/// Converts `self` to a [`KnString`].
	fn to_knstring(&self, env: &mut crate::Environment) -> crate::Result<KnString<'gc>>;
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
					alloc: Alloc { _padding: MaybeUninit::uninit(), ptr: $id.as_ptr(), len: $id.len() },
				},
			};
			KnString(&__INNER, PhantomData)
		}};
	}

	pub const TRUE: KnString<'_> = static_str!("true");
	pub const FALSE: KnString<'_> = static_str!("false");
}

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
	_padding: MaybeUninit<[u8; ALLOC_PADDING_ALIGN]>,
	ptr: *const u8,
	len: usize,
}

sa::const_assert_eq!(size_of::<Inner>(), ALLOC_VALUE_SIZE_IN_BYTES);
sa::assert_eq_size!(KnString, super::Value);

impl Default for KnString<'_> {
	#[inline]
	fn default() -> Self {
		static EMPTY_INNER: Inner = Inner {
			_alignment: ValueAlign,
			flags: AtomicU8::new(gc::FLAG_IS_STRING | gc::FLAG_GC_STATIC),
			kind: Kind { embedded: [0; MAX_EMBEDDED_LENGTH] },
		};

		Self(&EMPTY_INNER, PhantomData)
	}
}

impl<'gc> KnString<'gc> {
	/// Creates a new [`KnString`] from the given `source`.
	pub fn new(source: &KnStr, gc: &'gc Gc) -> Self {
		match source.len() {
			0 => Self::default(),

			// SAFETY: we know it's within the bounds because we checked in the `match`
			1..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(source.as_str(), gc) },

			_ => unsafe { Self::new_alloc(source.as_str(), gc) },
		}
	}

	pub(super) fn into_raw(self) -> *const ValueInner {
		self.0.cast()
	}

	pub(crate) unsafe fn from_raw(raw: *const ValueInner) -> Self {
		Self(raw.cast(), PhantomData)
	}

	// Allocate the underlying `ValueInner`.
	fn allocate(flags: u8, gc: &'gc Gc) -> *mut Inner {
		unsafe { gc.alloc_value_inner(gc::FLAG_IS_STRING as u8 | flags).cast::<Inner>() }
	}

	// SAFETY: `source.len()` needs to be `<= MAX_EMBEDDED_LENGTH`, otherwise we copy off the end.
	unsafe fn new_embedded(source: &str, gc: &'gc Gc) -> Self {
		let len = source.len();
		debug_assert!(len <= MAX_EMBEDDED_LENGTH);

		// Allocate the `Inner`.
		let inner = Self::allocate((len as u8) << SIZE_MASK_SHIFT, gc);

		// SAFETY:
		// - `Self::allocate` guarantees `(*inner).kind.embedded` is non-null and properly aligned
		let embedded_ptr = unsafe { (&raw mut (*inner).kind.embedded) }.cast::<u8>();

		// SAFETY
		// - caller guarantees that `source` has at least `len` bytes, so the `embedded_ptr` and
		//   `source.as_ptr()` are exactly `len` bytes.
		// - both are aligned for bytes.
		// - they don't overlap, as we just allocated the embedded pointer.
		unsafe {
			embedded_ptr.copy_from_nonoverlapping(source.as_ptr(), len);
		}

		Self(inner, PhantomData)
	}

	// SAFETY: source.len() cannot be zero
	unsafe fn new_alloc(source: &str, gc: &'gc Gc) -> Self {
		let len = source.len();
		debug_assert!(len > MAX_EMBEDDED_LENGTH, "too many bytes given; use new_embedded?");

		// Allocate the `Inner`.
		let inner = Self::allocate(ALLOCATED_FLAG, gc);

		// SAFETY: `Self::allocate` guarantees it'll be aligned and non-null
		unsafe {
			(&raw mut (*inner).kind.alloc.len).write(len);
		}

		// SAFETY:
		// - align `align_of::<u8>()` is nonzero and a power of two, as it's from `align_of::<u8>()`.
		// - size `len` came from a `&str`, we know it is `<= isize::MAX`
		let layout = unsafe { Layout::from_size_align_unchecked(len, align_of::<u8>()) };

		// SAFETY:
		// - `layout` is non-zero size, as caller guarantees it
		let alloc_ptr = unsafe { std::alloc::alloc(layout) };
		if alloc_ptr.is_null() {
			std::alloc::handle_alloc_error(layout);
		}

		// SAFETY:
		// - `alloc_ptr` was allocated specifically for `len`
		// - `source.as_ptr()` has exactly `len` bytes
		// - both are aligned for `u8`
		// - they don't overlap, as we just allocated `alloc_ptr`.
		unsafe {
			alloc_ptr.copy_from_nonoverlapping(source.as_ptr(), len);
		}

		// SAFETY: `Self::allocate` guarantees it'll be aligned and non-null
		unsafe {
			(&raw mut (*inner).kind.alloc.ptr).write(alloc_ptr);
		}

		Self(inner, PhantomData)
	}

	fn flags_and_inner(&self) -> (u8, *mut Inner) {
		unsafe {
			// TODO: orderings
			((*&raw const (*self.0).flags).load(Ordering::SeqCst), self.0 as _)
		}
	}

	/// Returns the underlying [`KnStr`].
	pub fn as_knstr(&self) -> &KnStr {
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

	pub fn len(self) -> usize {
		let (flags, inner) = self.flags_and_inner();

		if flags & ALLOCATED_FLAG as u8 != 0 {
			unsafe { (&raw const (*inner).kind.alloc.len).read() }
		} else {
			(flags as usize) >> SIZE_MASK_SHIFT
		}
	}
}

impl std::ops::Deref for KnString<'_> {
	type Target = KnStr;

	fn deref(&self) -> &Self::Target {
		self.as_knstr()
	}
}

impl Debug for KnString<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.as_knstr(), f)
	}
}

impl Display for KnString<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.as_knstr(), f)
	}
}

unsafe impl GarbageCollected for KnString<'_> {
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
