use std::cmp::Ordering;
use std::io::Write;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::gc::{GarbageCollected, GcRoot, ValueInner};
use crate::strings::KnStr;
use crate::{program::JumpIndex, vm::Vm, Environment, Error};

mod block;
mod boolean;
pub mod integer;
mod knstring;
mod list;
mod null;

pub use block::Block;
pub use boolean::{Boolean, ToBoolean};
pub use integer::{Integer, IntegerError, ToInteger};
pub use knstring::{KnString, ToKnString};
pub use list::{List, ToList};
pub use null::Null;
use std::fmt::{self, Debug, Formatter};
// pub use string::{KnString, ToKnString};

/// A trait indicating a type has a name.
pub trait NamedType {
	/// The name of a type.
	fn type_name(&self) -> &'static str;
}

/*
Representation:

0000 ... 0000 000 -- Null
0000 ... 0001 000 -- False
0000 ... 0010 000 -- True
XXXX ... XXXX 001 -- Integer
XXXX ... XXXX 010 -- Block
XXXX ... XXXX 100 -- Float32
XXXX ... XXXX 000 -- allocated, nonzero `X`
*/

/*
Representation (v2):

0000 ... 0000 000 -- Null
XXXX ... XXXX 000 -- "allocated", nonzero `X`
XXXX ... XXXX XX1 -- Integer
0000 ... 0000 010 -- False
0000 ... 0001 010 -- True
XXXX ... XXXX 100 -- Block
XXXX ... XXXX 100 -- Float32
*/

#[repr(transparent)] // DON'T DERIVE CLONE/COPY
#[derive(Clone, Copy)]
pub struct Value<'gc>(Inner, PhantomData<&'gc ()>);

#[repr(C)]
#[derive(Clone, Copy)]
union Inner {
	ptr: *const ValueInner,
	val: u64,
}

#[repr(align(16))]
pub(crate) struct ValueAlign;
sa::assert_eq_size!(ValueAlign, ());

// The amount of bytes expected in an allocated value
pub const ALLOC_VALUE_SIZE_IN_BYTES: usize = 32;
type ValueRepr = u64;

const TAG_SHIFT: ValueRepr = 4;
const TAG_MASK: ValueRepr = (1 << TAG_SHIFT) - 1;
const TAG_MASK2: ValueRepr = 0b111;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[rustfmt::skip]
enum Tag {
	// TOPMOST BIT IS Whether it's allocated
	Alloc          = 0b000,
	Integer        = 0b001,
	Block          = 0b010,
	Const          = 0b110,
	#[cfg(feature = "floats")]
	Float          = 0b100,
}

impl Debug for Value<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.is_null() {
			Debug::fmt(&Null, f)
		} else if let Some(boolean) = self.as_boolean() {
			Debug::fmt(&boolean, f)
		} else if let Some(integer) = self.as_integer() {
			Debug::fmt(&integer, f)
		} else if let Some(list) = self.as_list() {
			Debug::fmt(&list, f)
		} else if let Some(string) = self.as_knstring() {
			Debug::fmt(&string, f)
		} else if let Some(block) = self.as_block() {
			Debug::fmt(&block, f)
		} else {
			unreachable!()
		}
	}
}

// /// Returned when a `TryFrom` for a value was called when the type didnt match.
// #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct WrongType;

impl From<Null> for Value<'_> {
	#[inline]
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

impl From<Integer> for Value<'_> {
	#[inline]
	fn from(int: Integer) -> Self {
		let inner = int.inner();
		debug_assert_eq!((inner << 1) >> 1, inner);
		Self(Inner { val: ((inner as ValueRepr) << 1) | 1 }, PhantomData)
	}
}

impl From<Boolean> for Value<'_> {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		if boolean {
			Self::TRUE
		} else {
			Self::FALSE
		}
	}
}

impl From<Block> for Value<'_> {
	#[inline]
	fn from(block: Block) -> Self {
		let repr = block.inner().0 as u64;
		debug_assert!((repr << 3) >> 3 == repr, "repr has top 3 bits set");
		Self(Inner { val: (repr << 3) | 0b010 }, PhantomData)
	}
}

impl<'gc> From<List<'gc>> for Value<'gc> {
	#[inline]
	fn from(list: List) -> Self {
		unsafe { Self::from_alloc(list.into_raw()) }
	}
}

impl<'gc> From<KnString<'gc>> for Value<'gc> {
	#[inline]
	fn from(string: KnString) -> Self {
		sa::const_assert!(std::mem::size_of::<usize>() <= std::mem::size_of::<ValueRepr>());
		let raw = string.into_raw();
		unsafe { Self::from_alloc(raw) }
	}
}

impl NamedType for Value<'_> {
	/// Fetch the type's name.
	#[must_use = "getting the type name by itself does nothing."]
	fn type_name(&self) -> &'static str {
		if self.is_null() {
			Null.type_name()
		} else if let Some(x) = self.as_boolean() {
			x.type_name()
		} else if let Some(x) = self.as_integer() {
			x.type_name()
		} else if let Some(x) = self.as_knstring() {
			x.type_name()
		} else if let Some(x) = self.as_list() {
			x.type_name()
		} else if let Some(x) = self.as_block() {
			x.type_name()
		} else {
			todo!("other types")
		}
	}
}

impl<'gc> Value<'gc> {
	pub const NULL: Self = Self(Inner { val: 0b_0000_000 }, PhantomData);
	pub const FALSE: Self = Self(Inner { val: 0b_0000_010 }, PhantomData);
	pub const TRUE: Self = Self(Inner { val: 0b_0001_010 }, PhantomData);

	// SAFETY: bytes is a valid representation
	// #[inline]
	// const unsafe fn from_raw_shift(repr: ValueRepr, tag: Tag) -> Self {
	// 	debug_assert!((repr << TAG_SHIFT) >> TAG_SHIFT == repr, "repr has top TAG_SHIFT bits set");
	// 	Self(Inner { val: (repr << TAG_SHIFT) | tag as u64 }, PhantomData)
	// }

	// // SAFETY: bytes is a valid representation
	// #[inline]
	// const unsafe fn from_raw_shift_no_debug(repr: ValueRepr, tag: Tag) -> Self {
	// 	Self(Inner { val: (repr << TAG_SHIFT) | tag as u64 }, PhantomData)
	// }

	// SAFETY: bytes is a valid representation
	#[inline]
	unsafe fn from_alloc(ptr: *const ValueInner) -> Self {
		debug_assert!((ptr as usize) & (TAG_MASK as usize) == 0, "repr has tag bits set");
		Self(Inner { ptr }, PhantomData)
	}

	#[inline]
	pub(crate) unsafe fn __as_alloc(self) -> *const ValueInner {
		debug_assert!(self.is_alloc());
		unsafe { self.0.ptr }
	}

	const fn tag(self) -> Tag {
		let mask = unsafe { self.0.val & TAG_MASK } as u8;
		debug_assert!(
			mask == Tag::Alloc as _
				|| mask == Tag::Integer as _
				|| mask == Tag::Block as _
				|| mask == Tag::Const as _
		);
		unsafe { std::mem::transmute::<u8, Tag>(mask) }
	}

	fn is_alloc(self) -> bool {
		self.tag() == Tag::Alloc
	}

	fn is_alloc_or_null(self) -> bool {
		self.val() & TAG_MASK2 == 0
	}

	pub(crate) fn __is_alloc(self) -> bool {
		self.tag() == Tag::Alloc
	}

	const fn parts_shift(self) -> (ValueRepr, Tag) {
		(unsafe { self.0.val } >> TAG_SHIFT, self.tag())
	}

	pub const fn is_null(self) -> bool {
		unsafe { self.0.val == Self::NULL.0.val }
	}

	pub const fn as_integer(self) -> Option<Integer> {
		// Can't use `==` b/c the PartialEq impl isn't `const`.
		if !matches!(self.tag(), Tag::Integer) {
			return None;
		}

		// Can't use `parts_shift()` because it doesnt do sign-extending.
		Some(Integer::new_unvalidated_unchecked(
			unsafe { self.0.val as crate::value::integer::IntegerInner } >> TAG_SHIFT,
		))
	}

	pub const fn as_boolean(self) -> Option<Boolean> {
		unsafe {
			if self.0.val == Self::TRUE.0.val {
				Some(true)
			} else if self.0.val == Self::FALSE.0.val {
				Some(false)
			} else {
				None
			}
		}
	}

	pub fn as_block(self) -> Option<Block> {
		let (repr, tag) = self.parts_shift();

		matches!(tag, Tag::Block).then(|| Block::new(JumpIndex(repr as _)))
	}

	pub fn as_list(self) -> Option<List<'gc>> {
		if self.is_alloc() {
			unsafe { ValueInner::as_list(self.0.ptr) }
		} else {
			None
		}
	}

	pub fn as_knstring(self) -> Option<KnString<'gc>> {
		if self.is_alloc() {
			unsafe { ValueInner::as_knstring(self.0.ptr) }
		} else {
			None
		}
	}
}

unsafe impl GarbageCollected for Value<'_> {
	unsafe fn mark(&self) {
		if self.is_alloc() {
			unsafe { ValueInner::mark(self.0.ptr) }
		}
	}

	unsafe fn deallocate(self) {
		if self.is_alloc() {
			unsafe { ValueInner::deallocate(self.0.ptr, true) }
		}
	}
}

#[cfg(feature = "compliance")]
fn forbid_block_arguments(value: &Value, function: &'static str) -> crate::Result<()> {
	if value.as_block().is_some() {
		return Err(Error::TypeError { type_name: value.type_name(), function });
	}

	if let Some(list) = value.as_list() {
		for ele in list.iter() {
			forbid_block_arguments(&ele, function)?;
		}
	}

	Ok(())
}

impl<'gc> Value<'gc> {
	pub fn kn_dump(&self, env: &mut Environment<'gc>) -> crate::Result<()> {
		if self.is_null() {
			write!(env.output(), "null")
		} else if let Some(b) = self.as_boolean() {
			write!(env.output(), "{b}")
		} else if let Some(i) = self.as_integer() {
			write!(env.output(), "{i}")
		} else if let Some(s) = self.as_knstring() {
			write!(env.output(), "{:?}", s.as_str())
		} else if let Some(l) = self.as_list() {
			write!(env.output(), "[").map_err(|err| Error::IoError { func: "OUTPUT", err })?;
			for (idx, arg) in l.iter().enumerate() {
				if idx != 0 {
					write!(env.output(), ", ").map_err(|err| Error::IoError { func: "OUTPUT", err })?;
				}
				arg.kn_dump(env)?;
			}
			write!(env.output(), "]")
		} else {
			#[cfg(feature = "compliance")]
			if env.opts().compliance.cant_dump_blocks && self.as_block().is_some() {
				return write!(env.output(), "{:?}", self.as_block().unwrap())
					.map_err(|err| Error::IoError { func: "OUTPUT", err });
			}

			return Err(Error::TypeError { type_name: self.type_name(), function: "DUMP" });
		}
		.map_err(|err| Error::IoError { func: "OUTPUT", err })
	}

	pub fn kn_compare(
		&self,
		rhs: &Self,
		function: &'static str,
		env: &mut Environment<'gc>,
	) -> crate::Result<Ordering> {
		if let Some(integer) = self.as_integer() {
			return Ok(integer.cmp(&rhs.to_integer(env)?));
		}

		if let Some(string) = self.as_knstring() {
			return Ok(string.cmp(&rhs.to_knstring(env)?));
		}

		if let Some(boolean) = self.as_boolean() {
			return Ok(boolean.cmp(&rhs.to_boolean(env)?));
		}

		if let Some(list) = self.as_list() {
			return list.try_cmp(&*rhs.to_list(env)?, function, env);
		}

		Err(Error::TypeError { type_name: self.type_name(), function })
	}

	pub fn kn_equals(&self, rhs: &Self, env: &mut Environment<'gc>) -> crate::Result<bool> {
		// Rust's `==` semantics here actually directly map on to how equality in Knight works.

		// In strict compliance mode, we can't use Blocks for `?`.
		#[cfg(feature = "compliance")]
		if env.opts().compliance.check_equals_params {
			forbid_block_arguments(self, "?")?;
			forbid_block_arguments(rhs, "?")?;
		}

		let _ = env;
		Ok(self == rhs)
	}

	pub fn kn_call(&self, vm: &mut Vm<'_, '_, '_, '_, 'gc>) -> crate::Result<Self> {
		if let Some(block) = self.as_block() {
			vm.run(block)
		} else {
			Err(Error::TypeError { type_name: self.type_name(), function: "CALL" })
		}
	}

	// SAFETY: `target` has to be something which is garbage collected
	// (Note: current impl doesn't _actually_ require this, but this is future-compatibility)
	pub fn kn_length(&self, env: &mut Environment<'gc>) -> crate::Result<Integer> {
		if let Some(string) = self.as_knstring() {
			// Rust guarantees that `str::len` won't be larger than `isize::MAX`. Since we're always
			// using `i64`, if `usize == u32` or `usize == u64`, we can always cast the `isize` to
			// the `i64` without failure.
			//
			// With compliance enabled, it's possible that we are only checking for compliance on
			// integer bounds, and not on string lengths, so we do have to check in compliance mode.
			#[cfg(feature = "compliance")]
			if env.opts().compliance.i32_integer && !env.opts().compliance.check_container_length {
				return Ok(Integer::new_error(string.len() as i64, env.opts())?.into());
			}

			return Ok(Integer::new_unvalidated(string.len() as i64).into());
		}

		if let Some(list) = self.as_list() {
			// (same guarantees as `ValueEnum::String`)
			#[cfg(feature = "compliance")]
			if env.opts().compliance.i32_integer && !env.opts().compliance.check_container_length {
				return Ok(Integer::new_error(list.len() as i64, env.opts())?.into());
			}

			return Ok(Integer::new_unvalidated(list.len() as i64).into());
		}

		// TODO: optimizations of other things
		Ok(Integer::new_error(self.to_list(env)?.len() as i64, env.opts())?)
	}

	pub unsafe fn kn_not(
		&self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		target.write((!self.to_boolean(env)?).into());
		Ok(())
	}

	pub unsafe fn kn_negate(
		&self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		#[cfg(feature = "extensions")]
		if env.opts().extensions.breaking.negate_reverses_collections {
			todo!();
		}

		target.write(self.to_integer(env)?.negate(env.opts())?.into());
		Ok(())
	}

	// SAFETY: the target needs to be a gc-rooted place
	pub unsafe fn kn_plus(
		&self,
		rhs: &Self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(integer) = self.as_integer() {
			target.write(integer.add(rhs.to_integer(env)?, env.opts())?.into());
			return Ok(());
		}

		if let Some(string) = self.as_knstring() {
			let foo = string.concat(&rhs.to_knstring(env)?, env.opts(), env.gc())?;
			unsafe {
				foo.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		if let Some(list) = self.as_list() {
			let foo = list.concat(&*rhs.to_list(env)?, env.opts(), env.gc())?;
			unsafe {
				foo.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		#[cfg(feature = "extensions")]
		if env.opts().extensions.builtin_fns.boolean {
			if let Some(b) = self.as_boolean() {
				target.write((b | rhs.to_boolean(env)?).into());
				return Ok(());
			}
		}

		// 	#[cfg(feature = "custom-types")]
		// 	ValueEnum::Custom(custom) => custom.add(rhs, env),

		Err(Error::TypeError { type_name: self.type_name(), function: "+" })
	}

	pub unsafe fn kn_minus(
		&self,
		rhs: &Self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(integer) = self.as_integer() {
			target.write(integer.subtract(rhs.to_integer(env)?, env.opts())?.into());
			return Ok(());
		}

		#[cfg(feature = "extensions")]
		{
			if env.opts().extensions.builtin_fns.string {
				// return Ok(string.remove_substr(&rhs.to_kstring(env)?).into());
				todo!()
			}

			if env.opts().extensions.builtin_fns.list {
				// return list.difference(&rhs.to_list(env)?).map(Self::from);
				todo!()
			}
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "-" })
	}

	pub unsafe fn kn_asterisk(
		&self,
		rhs: &Self,
		target: &mut MaybeUninit<Value<'gc>>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(integer) = self.as_integer() {
			target.write(integer.multiply(rhs.to_integer(env)?, env.opts())?.into());
			return Ok(());
		}

		if let Some(string) = self.as_knstring() {
			let amount = usize::try_from(rhs.to_integer(env)?.inner())
				.or(Err(IntegerError::DomainError("repetition count is negative")))?;

			if amount.checked_mul(string.len()).map_or(true, |c| isize::MAX as usize <= c) {
				return Err(IntegerError::DomainError("repetition is too large").into());
			}

			let repeated = string.repeat(amount, env.opts(), env.gc())?;
			unsafe {
				repeated.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		if let Some(list) = self.as_list() {
			// Multiplying by a block is invalid, so we can do this as an extension.
			#[cfg(feature = "extensions")]
			if env.opts().extensions.builtin_fns.list && rhs.as_block().is_some() {
				// return list.map(rhs, env).map(Self::from);
				todo!()
			}

			let amount = usize::try_from(rhs.to_integer(env)?.inner())
				.or(Err(IntegerError::DomainError("repetition count is negative")))?;

			let repeated = list.repeat(amount, env.opts(), env.gc())?;
			unsafe {
				repeated.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "*" })
	}

	pub unsafe fn kn_slash(
		&self,
		rhs: &Self,
		target: &mut MaybeUninit<Value<'gc>>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(integer) = self.as_integer() {
			target.write(integer.divide(rhs.to_integer(env)?, env.opts())?.into());
			return Ok(());
		}

		#[cfg(feature = "extensions")]
		{
			if env.opts().extensions.builtin_fns.string {
				if let Some(string) = self.as_knstring() {
					let _ = string;
					// Ok(string.split(&rhs.to_kstring(env)?, env).into())
					todo!()
				}
			}

			if env.opts().extensions.builtin_fns.list {
				if let Some(list) = self.as_list() {
					let _ = list;
					// Ok(list.reduce(rhs, env)?.unwrap_or_default())
					todo!()
				}
			}
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "/" })
	}

	pub unsafe fn kn_percent(
		&self,
		rhs: &Self,
		target: &mut MaybeUninit<Value<'gc>>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(integer) = self.as_integer() {
			target.write(integer.remainder(rhs.to_integer(env)?, env.opts())?.into());
			return Ok(());
		}

		#[cfg(feature = "extensions")]
		{
			// TODO: `printf`-style formatting

			if env.opts().extensions.builtin_fns.list {
				if let Some(list) = self.as_list() {
					let _ = list;
					// list.filter(rhs, env).map(Self::from)
					todo!()
				}
			}
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "%" })
	}

	pub unsafe fn kn_caret(
		&self,
		rhs: &Self,
		target: &mut MaybeUninit<Value<'gc>>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(integer) = self.as_integer() {
			target.write(integer.power(rhs.to_integer(env)?, env.opts())?.into());
			return Ok(());
		}

		if let Some(list) = self.as_list() {
			let joined = list.join(&rhs.to_knstring(env)?, env)?;
			unsafe {
				joined.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "^" })
	}

	pub unsafe fn kn_head(
		&self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(string) = self.as_knstring() {
			let head = string.head(env.gc())?;
			unsafe {
				head.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		if let Some(list) = self.as_list() {
			target.write(list.get(0).ok_or(crate::Error::DomainError("empty list for head"))?);
			return Ok(());
		}

		#[cfg(feature = "extensions")]
		{
			if env.opts().extensions.builtin_fns.integer {
				if let Some(integer) = self.as_integer() {
					let _ = integer;
					// Ok(integer.head().into()),
					todo!()
				}
			}
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "[" })
	}

	pub unsafe fn kn_tail(
		&self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(string) = self.as_knstring() {
			let head = string.tail(env.gc())?;
			unsafe {
				head.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		if let Some(list) = self.as_list() {
			let head = list.tail(env.gc())?;
			unsafe {
				head.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		#[cfg(feature = "extensions")]
		{
			if env.opts().extensions.builtin_fns.integer {
				if let Some(integer) = self.as_integer() {
					let _ = integer;
					// Ok(integer.tail().into()),
					todo!()
				}
			}
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "]" })
	}

	pub unsafe fn kn_ascii(
		&self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		if let Some(integer) = self.as_integer() {
			let chr = integer.chr(env.opts())?;
			let mut buf = [0; 4];
			let gcstring = KnString::from_knstr(
				KnStr::new(chr.inner().encode_utf8(&mut buf), env.opts())?,
				&env.gc(),
			);

			unsafe {
				gcstring.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		if let Some(string) = self.as_knstring() {
			target.write(string.ord()?.into());
			return Ok(());
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "ASCII" })
	}

	pub unsafe fn kn_get(
		&self,
		start: &Self,
		len: &Self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		let start = fix_len(self, start.to_integer(env)?, "GET", env)?;
		let len = usize::try_from(len.to_integer(env)?.inner())
			.or(Err(Error::DomainError("negative length")))?;

		if let Some(list) = self.as_list() {
			let sublist = list.try_get(start..start + len, env.gc())?;
			unsafe {
				sublist.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}
		if let Some(string) = self.as_knstring() {
			let substring = string.try_get(start..start + len, env.gc())?;
			unsafe {
				substring.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "GET" })
	}

	pub unsafe fn kn_set(
		&self,
		start: &Self,
		len: &Self,
		repl: &Self,
		target: &mut MaybeUninit<Self>,
		env: &mut Environment<'gc>,
	) -> crate::Result<()> {
		#[cfg(feature = "custom-types")]
		{
			// if let ValueEnum::Custom(custom) = self {
			// 	return custom.set(start, len, replacement, env);
			// }
		}

		let start = fix_len(self, start.to_integer(env)?, "SET", env)?;
		let len = usize::try_from(len.to_integer(env)?.inner())
			.or(Err(Error::DomainError("negative length")))?;

		if let Some(list) = self.as_list() {
			let set = list.try_set(start, len, &*repl.to_list(env)?, env.opts(), env.gc())?;
			unsafe {
				set.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		if let Some(string) = self.as_knstring() {
			let set = string.try_set(start, len, &*repl.to_knstring(env)?, env.opts(), env.gc())?;
			unsafe {
				set.with_inner(|inner| target.write(inner.into()));
			}
			return Ok(());
		}

		Err(Error::TypeError { type_name: self.type_name(), function: "SET" })
	}

	fn val(&self) -> u64 {
		// safety: all permutations are valid `u64`s
		unsafe { self.0.val }
	}
}

fn fix_len(
	container: &Value<'_>,
	#[cfg_attr(not(feature = "extensions"), allow(unused_mut))] mut start: Integer,
	function: &'static str,
	env: &mut Environment<'_>,
) -> crate::Result<usize> {
	#[cfg(feature = "extensions")]
	if env.opts().extensions.negative_indexing && start < Integer::ZERO {
		let len = if let Some(string) = container.as_knstring() {
			string.len()
		} else if let Some(list) = container.as_list() {
			list.len()
		} else {
			return Err(Error::TypeError { type_name: container.type_name(), function });
		};

		start = start.add(Integer::new_error(len as _, env.opts())?, env.opts())?;
	}

	let _ = (container, env);
	usize::try_from(start.inner()).or(Err(Error::DomainError("negative start position")))
}

impl ToInteger for Value<'_> {
	fn to_integer(&self, env: &mut Environment<'_>) -> crate::Result<Integer> {
		// Special case for NULL, FALSE, and 0 based on their representations.
		if self.val() <= 0b10 {
			debug_assert!(
				self.is_null()
					|| self.as_boolean().map_or(false, |x| x == false)
					|| self.as_integer().map_or(false, |x| x == 0)
			);

			return Ok(Integer::new_unvalidated(0));
		}

		if let Some(boolean) = self.as_boolean() {
			debug_assert!(boolean, "the false case should've already been handled above");
			unsafe { std::hint::assert_unchecked(boolean) };
			return boolean.to_integer(env);
		}

		if let Some(integer) = self.as_integer() {
			debug_assert_ne!(integer, 0, "should've already been handled");
			return Ok(integer);
		}

		if let Some(list) = self.as_list() {
			return list.to_integer(env);
		}

		if let Some(string) = self.as_knstring() {
			return string.to_integer(env);
		}

		#[cfg(feature = "extensions")]
		{
			// TODO: check for `float`s
		}

		debug_assert!(self.as_block().is_some());

		if self.as_block().is_some() {
			return Err(crate::Error::Todo("cannot convert Blocks to integers".into()));
		}

		unsafe {
			unreachable_unchecked!("[BUG] invalid type for `to_integer()`?? {:?}", self.val());
		}
	}
}

impl ToBoolean for Value<'_> {
	fn to_boolean(&self, env: &mut Environment<'_>) -> crate::Result<Boolean> {
		// Special case for NULL, FALSE, and 0 based on their representations.
		if self.val() <= 0b10 {
			debug_assert!(
				self.is_null()
					|| self.as_boolean().map_or(false, |x| x == false)
					|| self.as_integer().map_or(false, |x| x == 0)
			);
			return Ok(false);
		}

		debug_assert!(!self.is_null());

		if !self.is_alloc_or_null() {
			#[cfg(feature = "extensions")]
			{
				// TODO: check for `float`s, and return possibly `false` for them.
			}

			#[cfg(debug_assertions)]
			if let Some(b) = self.as_boolean() {
				debug_assert!(b, "the false condition shoulda been checked earlier");
			} else if let Some(i) = self.as_integer() {
				debug_assert_ne!(i, 0, "the `zero` condition should've already been checked");
			} else {
				debug_assert!(self.as_block().is_some() /*|| self.as_float().is_some()*/);
			}

			#[cfg(feature = "compliance")]
			if env.opts().compliance.no_block_conversions && self.as_block().is_some() {
				return Err(crate::Error::Todo("cannot convert Blocks to booleans".into()));
			}

			return Ok(true);
		}

		if let Some(list) = self.as_list() {
			return Ok(list.is_empty());
		}

		if let Some(string) = self.as_knstring() {
			return Ok(string.is_empty());
		}

		// SAFETY: we've already covered every single type, so there's no reason this should ever
		// happen.
		unsafe {
			unreachable_unchecked!("[BUG] invalid type for `to_boolean()`?? {:?}", self.val());
		}
	}
}

impl<'gc> ToKnString<'gc> for Value<'gc> {
	fn to_knstring(&self, env: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, KnString<'gc>>> {
		if self.val() <= knstring::consts::LITERAL_MAX_LENGTH as _ {
			#[cfg(feature = "compliance")]
			if env.opts().compliance.no_block_conversions && self.as_block().is_some() {
				return Err(crate::Error::Todo("cannot convert Blocks to strings".into()));
			}

			// NOTE: We need to somehow guarantee that we'll never actually pass in pointers
			// `0b01_000` or `0b10_000`.
			debug_assert!(
				self.is_null()
					|| self.as_boolean().is_some()
					|| self.as_integer().map_or(false, |i| i <= 9)
			);

			return Ok(GcRoot::new_unchecked(unsafe {
				knstring::consts::lookup_literal(self.val() as _)
			}));
		}

		if let Some(string) = self.as_knstring() {
			return string.to_knstring(env);
		}

		if let Some(list) = self.as_list() {
			return list.to_knstring(env);
		}

		if let Some(integer) = self.as_integer() {
			return integer.to_knstring(env);
		}

		#[cfg(feature = "extensions")]
		{
			// TODO: check for `float`s
		}

		if self.as_block().is_some() {
			return Err(crate::Error::Todo("cannot convert Blocks to strings".into()));
		}

		unsafe {
			unreachable_unchecked!("[BUG] invalid type for `to_knstring()`?? {:?}", self.val());
		}
	}
}

impl<'gc> ToList<'gc> for Value<'gc> {
	fn to_list(&self, env: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, List<'gc>>> {
		// TODO: optimize me
		if let Some(list) = self.as_list() {
			return list.to_list(env);
		}

		if let Some(string) = self.as_knstring() {
			return string.to_list(env);
		}

		if let Some(integer) = self.as_integer() {
			return integer.to_list(env);
		}

		if let Some(boolean) = self.as_boolean() {
			return boolean.to_list(env);
		}

		if self.is_null() {
			return Null.to_list(env);
		}

		// todo: floats
		if self.as_block().is_some() {
			return Err(crate::Error::Todo("cannot convert Blocks to lists".into()));
		}

		unsafe {
			unreachable_unchecked!("[BUG] invalid type for `to_list()`?? {:?}", self.val());
		}
	}
}

impl PartialEq for Value<'_> {
	fn eq(&self, rhs: &Self) -> bool {
		if unsafe { self.0.val == rhs.0.val } {
			return true;
		}

		if !self.is_alloc() || !rhs.is_alloc() {
			return false;
		}

		if let Some(knstr) = self.as_knstring() {
			rhs.as_knstring().map_or(false, |r| knstr == r)
		} else if let Some(list) = self.as_list() {
			rhs.as_list().map_or(false, |r| list == r)
		} else {
			unreachable!()
		}
	}
}
