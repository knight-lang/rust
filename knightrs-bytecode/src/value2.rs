use std::cmp::Ordering;

use crate::{program::JumpIndex, vm::Vm, Environment, Error};

mod block;
mod boolean;
mod integer;
mod knstring;
mod list;
mod null;

pub use block::Block;
pub use boolean::{Boolean, ToBoolean};
pub use integer::{Integer, IntegerError, ToInteger};
pub use knstring::KnString;
pub use list::List;
pub use null::Null;
use std::fmt::{self, Debug, Formatter};
// pub use string::{KnValueString, ToKnValueString};

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
XXXX ... XXXX 010 -- String
XXXX ... XXXX 100 -- List
XXXX ... XXXX 110 -- Block
XXXX ... XXXX 111 -- Custom
*/
#[repr(transparent)] // DON'T DERIVE CLONE/COPY
pub struct Value(ValueRepr);

#[repr(align(16))]
struct ValueAlign;
sa::assert_eq_size!(ValueAlign, ());

// The amount of bytes expected in an allocated value
pub const ALLOC_VALUE_SIZE_IN_BYTES: usize = 32;
type ValueRepr = u64;

const TAG_SHIFT: ValueRepr = 4;
const TAG_MASK: ValueRepr = (1 << TAG_SHIFT) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[rustfmt::skip]
enum Tag {
	// TOPMOST BIT IS Whether it's allocated
	Const   = 0b000,
	Integer = 0b001,
	Block   = 0b010,
	#[cfg(feature = "floats")]
	Float   = 0b011,
	String  = 0b100,
	List    = 0b101,
	#[cfg(feature = "custom-types")]
	Custom  = 0b110,
}

impl Tag {
	fn is_alloc(self) -> bool {
		(self as u8) & 0b100 != 0
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self.tag() {
			Tag::Const => {
				if self.is_null() {
					Debug::fmt(&Null, f)
				} else if let Some(boolean) = self.as_boolean() {
					Debug::fmt(&boolean, f)
				} else {
					unreachable!()
				}
			}
			Tag::Integer => Debug::fmt(&self.as_integer().unwrap(), f),
			Tag::String => Debug::fmt(&self.as_knstring().unwrap(), f),
			Tag::List => todo!(), //Debug::fmt(&self.as_list().unwrap(), f),
			_ => todo!(),
		}
	}
}

// impl Drop for Value {
// 	fn drop(&mut self) {
// 		let (repr, tag) = self.parts_shift();
// 		if !tag.is_alloc() {
// 			return;
// 		}

// 		match tag {
// 			Tag::String => todo!(),
// 			Tag::List => todo!(), //drop(unsafe { List::from_raw(repr) }),
// 			#[cfg(feature = "custom-types")]
// 			Tag::Custom => todo!(),
// 			_ => unreachable!(),
// 		}
// 	}
// }

// /// Returned when a `TryFrom` for a value was called when the type didnt match.
// #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct WrongType;

impl From<Integer> for Value {
	#[inline]
	fn from(int: Integer) -> Self {
		unsafe { Self::from_raw_shift(int.inner() as ValueRepr, Tag::Integer) }
	}
}

impl From<Boolean> for Value {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		if boolean {
			Self::TRUE
		} else {
			Self::FALSE
		}
	}
}

impl From<Block> for Value {
	#[inline]
	fn from(block: Block) -> Self {
		unsafe { Self::from_raw_shift(block.inner().0 as ValueRepr, Tag::Block) }
	}
}

impl From<List> for Value {
	#[inline]
	fn from(list: List) -> Self {
		unsafe { Self::from_raw(list.into_raw(), Tag::List) }
	}
}

impl From<KnString> for Value {
	#[inline]
	fn from(string: KnString) -> Self {
		sa::const_assert!(std::mem::size_of::<usize>() <= std::mem::size_of::<ValueRepr>());
		let raw = string.into_raw();
		unsafe { Self::from_raw(raw, Tag::String) }
	}
}

impl Value {
	pub const FALSE: Self = unsafe { Self::from_raw_shift(0, Tag::Const) };
	pub const NULL: Self = unsafe { Self::from_raw_shift(1, Tag::Const) };
	pub const TRUE: Self = unsafe { Self::from_raw_shift(2, Tag::Const) };

	// SAFETY: bytes is a valid representation
	#[inline]
	const unsafe fn from_raw_shift(repr: ValueRepr, tag: Tag) -> Self {
		debug_assert!((repr << TAG_SHIFT) >> TAG_SHIFT == repr, "repr has top TAG_SHIFT bits set");
		unsafe { Self::from_raw(repr << TAG_SHIFT, tag) }
	}

	// SAFETY: bytes is a valid representation
	#[inline]
	const unsafe fn from_raw(repr: ValueRepr, tag: Tag) -> Self {
		debug_assert!(repr & TAG_MASK == 0, "repr has tag bits set");
		Self(repr | tag as ValueRepr)
	}

	const fn tag(&self) -> Tag {
		let mask = (self.0 & TAG_MASK) as u8;
		debug_assert!(
			mask == Tag::Const as _
				|| mask == Tag::Integer as _
				|| mask == Tag::String as _
				|| mask == Tag::List as _
				|| mask == Tag::Block as _
				|| {
					#[cfg(feature = "floats")]
					{
						mask == Tag::Float as _
					}
					#[cfg(not(feature = "custom-types"))]
					false
				} || {
				#[cfg(feature = "custom-types")]
				{
					mask == Tag::Custom as _
				}
				#[cfg(not(feature = "custom-types"))]
				false
			}
		);
		unsafe { std::mem::transmute::<u8, Tag>(mask) }
	}

	/// Get the bytes of `self` _WITHOUT RUNNING THE DESTRUCTOR_.
	const fn bytes(self) -> ValueRepr {
		let bytes = self.0;
		std::mem::forget(self);
		bytes
	}

	const fn parts_shift(&self) -> (ValueRepr, Tag) {
		(self.0 >> TAG_SHIFT, self.tag())
	}

	const fn parts(&self) -> (ValueRepr, Tag) {
		(self.0 & !TAG_MASK, self.tag())
	}

	pub const fn is_null(&self) -> bool {
		self.0 == Self::NULL.bytes()
	}

	pub const fn as_integer(&self) -> Option<Integer> {
		// Can't use `==` b/c the PartialEq impl isn't `const`.
		if !matches!(self.tag(), Tag::Integer) {
			return None;
		}

		// Can't use `parts_shift()` because it doesnt do sign-extending.
		Some(Integer::new_unvalidated_unchecked(
			(self.0 as crate::value::integer::IntegerInner) >> TAG_SHIFT,
		))
	}

	pub const fn as_boolean(&self) -> Option<Boolean> {
		if self.0 == Self::TRUE.bytes() {
			Some(true)
		} else if self.0 == Self::FALSE.bytes() {
			Some(false)
		} else {
			None
		}
	}

	pub fn as_block(&self) -> Option<Block> {
		let (repr, tag) = self.parts_shift();

		matches!(tag, Tag::Block).then(|| Block::new(JumpIndex(tag as _)))
	}

	pub fn as_list(&self) -> Option<List> {
		let (repr, tag) = self.parts_shift();

		matches!(tag, Tag::List).then(|| unsafe { List::from_raw_and_incr(repr) })
	}

	pub fn into_list(self) -> Option<List> {
		let (repr, tag) = self.parts();

		if !matches!(tag, Tag::List) {
			return None;
		}

		std::mem::forget(self);
		Some(unsafe { List::from_raw_and_incr(repr) })
	}

	pub fn as_knstring(&self) -> Option<KnString> {
		let (repr, tag) = self.parts();
		println!("{:0b}", repr);

		matches!(tag, Tag::String).then(|| unsafe { KnString::from_raw(repr as _) })
	}
}
