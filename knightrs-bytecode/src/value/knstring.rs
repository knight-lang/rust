use crate::gc::{self, AsValueInner, GarbageCollected, Gc, GcRoot, ValueInner};
use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::Compilable;
use crate::program::Compiler;
use crate::value::{Boolean, Integer, List, NamedType, ToBoolean, ToInteger, ToList};
use crate::{Environment, Options};
use std::alloc::Layout;
use std::fmt::{self, Debug, Display, Formatter};
use std::isize;
use std::marker::PhantomData;
use std::mem::{align_of, size_of, transmute, ManuallyDrop, MaybeUninit};
use std::slice::SliceIndex;
use std::sync::atomic::{AtomicU8, Ordering};

use super::{ValueAlign, ALLOC_VALUE_SIZE_IN_BYTES};
use crate::strings::{KnStr, StringError};

/// A KnString represents an allocated string within Knight, and is garbage collected.
///
/// (It's `Kn` because `String` is already a type in Rust, and I didn't want confusion.)
#[repr(transparent)]
pub struct KnString<'gc>(*const Inner, PhantomData<&'gc ()>);

/// Represents the ability to be converted to a [`KnString`].
pub trait ToKnString<'gc> {
	/// Converts `self` to a [`KnString`].
	fn to_knstring(
		&self,
		env: &mut crate::Environment<'gc>,
	) -> crate::Result<GcRoot<'gc, KnString<'gc>>>;
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

impl Eq for KnString<'_> {}
impl PartialEq for KnString<'_> {
	fn eq(&self, rhs: &Self) -> bool {
		self.as_str() == rhs.as_str()
	}
}

impl PartialEq<KnStr> for KnString<'_> {
	fn eq(&self, rhs: &KnStr) -> bool {
		self.as_str() == rhs.as_str()
	}
}

impl<'gc> KnString<'gc> {
	/// Creates a new [`KnString`] from the given `source`.
	pub fn from_knstr(source: &KnStr, gc: &'gc Gc) -> GcRoot<'gc, Self> {
		match source.len() {
			0 => GcRoot::new_unchecked(Self::default()),

			// SAFETY: we know it's within the bounds because we checked in the `match`
			1..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(source.as_str(), gc) },

			_ => unsafe { Self::new_alloc(source.to_string(), gc) },
		}
	}

	pub fn new(
		source: String,
		opts: &Options,
		gc: &'gc Gc,
	) -> Result<GcRoot<'gc, Self>, StringError> {
		KnStr::new(&source, opts)?;
		Ok(Self::new_unvalidated(source, gc))
	}

	pub fn new_unvalidated(source: String, gc: &'gc Gc) -> GcRoot<'gc, Self> {
		if source.is_empty() {
			return GcRoot::new_unchecked(Self::default());
		}

		// We already are given an allocated pointer, might as well use `new_alloc`
		unsafe { Self::new_alloc(source, gc) }
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
	unsafe fn new_embedded(source: &str, gc: &'gc Gc) -> GcRoot<'gc, Self> {
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

		GcRoot::new(&Self(inner, PhantomData), gc)
	}

	// SAFETY: source.len() cannot be zero
	unsafe fn new_alloc(mut source: String, gc: &'gc Gc) -> GcRoot<'gc, Self> {
		let len = source.len();

		// Allocate the `Inner`.
		let inner = Self::allocate(ALLOCATED_FLAG, gc);

		// SAFETY: `Self::allocate` guarantees it'll be aligned and non-null
		unsafe {
			(&raw mut (*inner).kind.alloc.len).write(len);
		}

		source.shrink_to_fit();

		// SAFETY: `Self::allocate` guarantees it'll be aligned and non-null
		unsafe {
			(&raw mut (*inner).kind.alloc.ptr).write(ManuallyDrop::new(source).as_mut_ptr());
		}

		GcRoot::new(&Self(inner, PhantomData), gc)
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

	pub fn len(&self) -> usize {
		let (flags, inner) = self.flags_and_inner();

		if flags & ALLOCATED_FLAG as u8 != 0 {
			unsafe { (&raw const (*inner).kind.alloc.len).read() }
		} else {
			(flags as usize) >> SIZE_MASK_SHIFT
		}
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn concat(
		&self,
		other: &KnStr,
		opts: &Options,
		gc: &'gc Gc,
	) -> crate::Result<GcRoot<'gc, Self>> {
		let mut me = self.as_str().to_owned();
		me += other.as_str();
		Ok(Self::new(me, opts, gc)?)
	}

	pub fn repeat(
		&self,
		amount: usize,
		opts: &Options,
		gc: &'gc Gc,
	) -> crate::Result<GcRoot<'gc, Self>> {
		if self.len().checked_mul(amount).map_or(true, |f| f > isize::MAX as usize) {
			return Err(crate::Error::Todo("bounds too large!".to_string()));
		}

		if amount == 0 || self.is_empty() {
			return Ok(GcRoot::new_unchecked(Self::default()));
		}

		// todo: optimized variant?
		Ok(Self::new(self.as_str().repeat(amount), opts, gc)?)
	}

	#[cfg(feature = "extensions")]
	pub fn split(&self, by: &str, gc: &'gc Gc) -> crate::Result<GcRoot<'gc, List<'gc>>> {
		// let list =
		// Ok(List::new_unvalidated(self.as_str().split(by).map(|substr|collect(), gc))
		todo!()
	}

	pub fn head(&self, gc: &'gc Gc) -> crate::Result<GcRoot<'gc, Self>> {
		let mut buf = [0; 4];
		let head_string = self
			.as_str()
			.chars()
			.next()
			.ok_or(crate::Error::DomainError("empty string for head"))?
			.encode_utf8(&mut buf);

		Ok(Self::from_knstr(KnStr::new_unvalidated(head_string), gc))
	}

	pub fn tail(&self, gc: &'gc Gc) -> crate::Result<GcRoot<'gc, Self>> {
		let mut chars = self.as_str().chars();
		if chars.next().is_none() {
			return Err(crate::Error::DomainError("empty string for tail"));
		}

		Ok(Self::from_knstr(KnStr::new_unvalidated(chars.as_str()), gc))
	}

	pub fn ord(&self) -> crate::Result<Integer> {
		let first =
			self.as_str().chars().next().ok_or(crate::Error::DomainError("empty string for head"))?;

		Ok(Integer::new_unvalidated(first as _))
	}

	pub fn try_get<I>(&self, index: I, gc: &'gc Gc) -> crate::Result<GcRoot<'gc, Self>>
	where
		I: SliceIndex<str, Output = str>,
	{
		let rest = self
			.as_str()
			.get(index)
			.ok_or(crate::Error::DomainError("invalid args for get for str"))?;

		Ok(Self::from_knstr(KnStr::new_unvalidated(rest), gc))
	}

	pub fn try_set(
		&self,
		start: usize,
		len: usize,
		repl: &KnStr,
		opts: &Options,
		gc: &'gc Gc,
	) -> crate::Result<GcRoot<'gc, Self>> {
		// TODO: optimize this
		let mut s = String::new();
		let mut chars = self.as_str().chars();

		s.push_str(&chars.by_ref().take(start).collect::<String>());
		s.push_str(repl.as_str());
		s.push_str(&chars.skip(len).collect::<String>());
		Ok(Self::new(s, opts, gc)?)
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
			let ptr = (&raw mut (*inner).kind.alloc.ptr).read() as *mut u8;
			let len = (&raw mut (*inner).kind.alloc.len).read();
			drop(String::from_raw_parts(ptr, len, len));
		}
	}
}

unsafe impl<'gc> AsValueInner for KnString<'gc> {
	fn as_value_inner(&self) -> *const ValueInner {
		self.0.cast()
	}

	unsafe fn from_value_inner(inner: *const ValueInner) -> Self {
		unsafe { Self::from_raw(inner) }
	}
}

impl NamedType for KnString<'_> {
	#[inline]
	fn type_name(&self) -> &'static str {
		"String"
	}
}

impl ToBoolean for KnString<'_> {
	/// Simply returns `self`.
	#[inline]
	fn to_boolean(&self, _: &mut Environment<'_>) -> crate::Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToInteger for KnString<'_> {
	/// Returns `1` for true and `0` for false.
	#[inline]
	fn to_integer(&self, env: &mut Environment<'_>) -> crate::Result<Integer> {
		Integer::parse_from_str(self.as_str(), env.opts())
	}
}

impl<'gc> ToKnString<'gc> for KnString<'gc> {
	/// Returns `"true"` for true and `"false"` for false.
	#[inline]
	fn to_knstring(&self, _: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, KnString<'gc>>> {
		// Since `self` is already a part of the gc, then cloning it does nothing.
		Ok(GcRoot::new_unchecked(Self(self.0, PhantomData)))
	}
}

impl<'gc> ToList<'gc> for KnString<'gc> {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	#[inline]
	fn to_list(&self, env: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, List<'gc>>> {
		env.gc().pause();

		let chars = self
			.chars()
			.map(|c| {
				let chr_string = Self::new_unvalidated(c.to_string(), env.gc());
				unsafe { chr_string.assume_used() }.into()
			})
			.collect::<Vec<_>>();

		// COMPLIANCE: If `self` is within the container bounds, so is the length of its chars.
		let result = List::new_unvalidated(chars, env.gc());
		env.gc().unpause();

		Ok(result)
	}
}

impl<'path, 'gc> Parseable<'_, 'path, 'gc> for KnString<'gc> {
	type Output = GcRoot<'gc, Self>;

	fn parse(
		parser: &mut Parser<'_, '_, 'path, 'gc>,
	) -> Result<Option<Self::Output>, ParseError<'path>> {
		#[cfg(feature = "extensions")]
		if parser.opts().extensions.syntax.string_interpolation && parser.advance_if('`').is_some() {
			todo!();
		}

		let Some(quote) = parser.advance_if(|c| c == '\'' || c == '\"') else {
			return Ok(None);
		};

		let start = parser.location();

		// empty stings are allowed to exist
		let contents = parser.take_while(|c| c != quote).unwrap_or_default();

		if parser.advance_if(quote).is_none() {
			return Err(start.error(ParseErrorKind::MissingEndingQuote(quote)));
		}

		let string = KnString::new(contents.to_string(), parser.opts(), parser.gc())
			.map_err(|err| start.error(err.into()))?;
		Ok(Some(string))
	}
}

unsafe impl<'path, 'gc> Compilable<'_, 'path, 'gc> for GcRoot<'gc, KnString<'gc>> {
	fn compile(
		self,
		compiler: &mut Compiler<'_, 'path, 'gc>,
		_: &Options,
	) -> Result<(), ParseError<'path>> {
		// TODO: SAFETY CHECK: compielr must have a reference to `self`
		unsafe {
			self.with_inner(|inner| compiler.push_constant(inner.into()));
		}
		Ok(())
	}
}
