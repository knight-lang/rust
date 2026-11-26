use crate::gc::{self, AsValueInner, GarbageCollected, Gc, GcRoot, ValueInner};
use crate::parser::{ParseError, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::KnStr;
use crate::value::List;
use crate::value::ValueAlign;
use crate::value::ALLOC_VALUE_SIZE_IN_BYTES;
use crate::value::{Boolean, Integer, KnString, NamedType, ToBoolean, ToInteger, ToKnString};
use crate::Value;
use crate::{Environment, Error, Options};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::mem::{size_of, ManuallyDrop, MaybeUninit};
use std::slice::SliceIndex;
use std::sync::atomic::AtomicU8;

pub trait CustomType {}

sa::assert_obj_safe!(CustomType);

#[repr(transparent)]
pub struct Custom<'gc>(*const Inner<'gc>);

#[repr(C)]
struct Inner<'gc> {
	_alignment: ValueAlign,
	flags: AtomicU8,
	_align: MaybeUninit<[u8; 7]>, // TODO: don't use a constant
	_dont_need_gc_rn: &'gc (),
	body: [u8],
}

// sa::const_assert_eq!(size_of::<Inner<'_>>(), ALLOC_VALUE_SIZE_IN_BYTES);
// sa::assert_eq_size!(List, super::Value);

impl<'gc> Custom<'gc> {
	pub fn new<T: CustomType>(data: T, gc: &'gc Gc) -> GcRoot<'gc, Self> {
		let inner = unsafe { gc.alloc_value_inner(flags | gc::FLAG_IS_CUSTOM) }.cast::<Inner>();

		match source.len() {
			0 => GcRoot::new_unchecked(Self::default()),
			//TODO
			1..=MAX_EMBEDDED_LENGTH => unsafe { Self::new_embedded(&source.collect::<Vec<_>>(), gc) },
			_ => Self::new_alloc(source.collect(), gc),
		}
	}
}
