use crate::container::RefCount;
use crate::gc::{self, AsValueInner, GarbageCollected, Gc, GcRoot, ValueInner};
use crate::parser::ParseError;
use crate::program::Compilable;
use crate::program::Compiler;
use crate::strings::KnStr;
use crate::value::{Boolean, Integer, KnString, NamedType, ToBoolean, ToInteger, ToKnString};
use crate::{Environment, Error, Options};
use std::alloc::Layout;
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem::{align_of, size_of, transmute, ManuallyDrop, MaybeUninit};
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
		_align: MaybeUninit::uninit(),
		kind: Kind { embedded: [Value::TRUE; MAX_EMBEDDED_LENGTH] },
	};
}

/// Represents the ability to be converted to a [`List`].
pub trait ToList<'gc> {
	/// Converts `self` to a [`List`].
	fn to_list(&self, env: &mut crate::Environment<'gc>) -> crate::Result<GcRoot<'gc, List<'gc>>>;
}

#[repr(C)]
struct Inner<'gc> {
	_alignment: ValueAlign,
	flags: AtomicU8,
	_align: MaybeUninit<[u8; 7]>, // TODO: don't use a constant
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

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Alloc<'gc> {
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
			_align: MaybeUninit::uninit(),
			kind: Kind { embedded: [Value::NULL; MAX_EMBEDDED_LENGTH] },
		};
		Self(&EMPTY_INNER)
	}
}
impl<'gc> List<'gc> {
	/// The maximum length a list can be when compliance checking is enabled.
	pub const COMPLIANCE_MAX_LEN: usize = i32::MAX as usize;

	pub fn into_raw(self) -> *const ValueInner {
		self.0.cast()
	}

	pub unsafe fn from_raw(ptr: *const ValueInner) -> Self {
		Self(ptr.cast())
	}

	pub fn boxed(value: Value<'gc>, gc: &'gc Gc) -> GcRoot<'gc, Self> {
		Self::from_slice_unvalidated(&[value], gc)
	}

	pub fn from_slice(
		source: &[Value<'gc>],
		opts: &Options,
		gc: &'gc Gc,
	) -> crate::Result<GcRoot<'gc, Self>> {
		#[cfg(feature = "compliance")]
		if opts.compliance.check_container_length && Self::COMPLIANCE_MAX_LEN < source.len() {
			return Err(Error::ListIsTooLarge);
		}

		Ok(Self::from_slice_unvalidated(source, gc))
	}

	pub fn from_slice_unvalidated(source: &[Value<'gc>], gc: &'gc Gc) -> GcRoot<'gc, Self> {
		match source.len() {
			0 => GcRoot::new_unchecked(Self::default()),
			1..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(source, gc) },
			_ => Self::new_alloc(source.to_vec(), gc),
		}
	}

	pub fn new(
		source: Vec<Value<'gc>>,
		opts: &Options,
		gc: &'gc Gc,
	) -> crate::Result<GcRoot<'gc, Self>> {
		#[cfg(feature = "compliance")]
		if opts.compliance.check_container_length && Self::COMPLIANCE_MAX_LEN < source.len() {
			return Err(Error::ListIsTooLarge);
		}

		Ok(Self::new_unvalidated(source, gc))
	}

	pub fn new_unvalidated(source: Vec<Value<'gc>>, gc: &'gc Gc) -> GcRoot<'gc, Self> {
		if source.is_empty() {
			return GcRoot::new_unchecked(Self::default());
		}

		// We already are given an allocated pointer, might as well use `new_alloc`
		Self::new_alloc(source, gc)
	}

	fn allocate(flags: u8, gc: &'gc Gc) -> *mut Inner<'gc> {
		unsafe { gc.alloc_value_inner(flags | gc::FLAG_IS_LIST) }.cast::<Inner>()
	}

	fn new_embedded(source: &[Value<'gc>], gc: &'gc Gc) -> GcRoot<'gc, Self> {
		debug_assert!(source.len() <= MAX_EMBEDDED_LENGTH);
		let inner = Self::allocate((source.len() as u8) << SIZE_MASK_SHIFT, gc);

		unsafe {
			(&raw mut (*inner).kind.embedded)
				.cast::<Value<'gc>>()
				.copy_from_nonoverlapping(source.as_ptr(), source.len());
		}

		GcRoot::new(&Self(inner), gc)
	}

	fn new_alloc(mut source: Vec<Value<'gc>>, gc: &'gc Gc) -> GcRoot<'gc, Self> {
		debug_assert!(source.len() > MAX_EMBEDDED_LENGTH);

		let inner = Self::allocate(ALLOCATED_FLAG, gc);

		source.shrink_to_fit();

		unsafe {
			(&raw mut (*inner).kind.alloc.len).write(source.len());
			(&raw mut (*inner).kind.alloc.ptr).write(ManuallyDrop::new(source).as_mut_ptr());
		}

		GcRoot::new(&Self(inner), gc)
	}

	fn flags_and_inner(&self) -> (u8, *mut Inner<'gc>) {
		unsafe {
			// TODO: orderings
			((*&raw const (*self.0).flags).load(std::sync::atomic::Ordering::Relaxed), self.0 as _)
		}
	}

	#[deprecated] // won't work with non-slice types
	fn as_slice<'e>(&'e self) -> &'e [Value<'gc>] {
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

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn join(
		&self,
		sep: &KnStr,
		env: &mut Environment<'gc>,
	) -> crate::Result<GcRoot<'gc, KnString<'gc>>> {
		let mut s = String::new();
		let mut first = true;

		let slice = self.as_slice();
		for element in slice {
			if first {
				first = false;
			} else {
				s.push_str(sep.as_str());
			}

			let ele_str = element.to_knstring(env)?;

			unsafe {
				ele_str.with_inner(|inner| s.push_str(inner.as_str()));
			}
		}
		Ok(KnString::new(s, env.opts(), env.gc())?)
		// // Ok(GcRoot::new_unchecked(Self(self.0, PhantomData)))
		// env.gc().pause();

		// let chars = self
		// 	.chars()
		// 	.map(|c| {
		// 		let chr_string = Self::new_unvalidated(c.to_string(), env.gc());
		// 		unsafe { chr_string.assume_used() }.into()
		// 	})
		// 	.collect::<Vec<_>>();

		// // COMPLIANCE: If `self` is within the container bounds, so is the length of its chars.
		// let result = List::new_unvalidated(chars, env.gc());
		// env.gc().unpause();

		// Ok(result)
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
			let ptr = (&raw mut (*inner).kind.alloc.ptr).read() as *mut Value<'_>;
			let len = (&raw mut (*inner).kind.alloc.len).read();

			drop(Vec::from_raw_parts(ptr, len, len));
		}
	}
}

unsafe impl<'gc> AsValueInner for List<'gc> {
	fn as_value_inner(&self) -> *const ValueInner {
		self.0.cast()
	}

	unsafe fn from_value_inner(inner: *const ValueInner) -> Self {
		unsafe { Self::from_raw(inner) }
	}
}

impl NamedType for List<'_> {
	#[inline]
	fn type_name(&self) -> &'static str {
		"List"
	}
}

impl ToBoolean for List<'_> {
	/// Simply returns `self`.
	#[inline]
	fn to_boolean(&self, _: &mut Environment<'_>) -> crate::Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToInteger for List<'_> {
	/// Returns `1` for true and `0` for false.
	#[inline]
	fn to_integer(&self, _: &mut Environment<'_>) -> crate::Result<Integer> {
		Ok(Integer::new_unvalidated(self.len() as _))
	}
}

impl<'gc> ToKnString<'gc> for List<'gc> {
	/// Returns `"true"` for true and `"false"` for false.
	#[inline]
	fn to_knstring(&self, env: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, KnString<'gc>>> {
		self.join(KnStr::new_unvalidated("\n"), env)
	}
}

impl<'gc> ToList<'gc> for List<'gc> {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	#[inline]
	fn to_list(&self, env: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, List<'gc>>> {
		// Since `self` is already a part of the gc, then cloning it does nothing.
		Ok(GcRoot::new_unchecked(Self(self.0)))
	}
}

// impl<'path> Parseable<'_, 'path> for Boolean {
// 	type Output = Self;

// 	fn parse(parser: &mut Parser<'_, '_, 'path, '_>) -> Result<Option<Self::Output>, ParseError<'path>> {
// 		let Some(chr) = parser.advance_if(|c| c == 'T' || c == 'F') else {
// 			return Ok(None);
// 		};

// 		parser.strip_keyword_function();
// 		Ok(Some(chr == 'T'))
// 	}
// }

unsafe impl<'gc, 'path> Compilable<'_, 'path, 'gc> for List<'gc> {
	fn compile(
		self,
		compiler: &mut Compiler<'_, 'path, 'gc>,
		_: &Options,
	) -> Result<(), ParseError<'path>> {
		compiler.push_constant(self.into());
		Ok(())
	}
}
