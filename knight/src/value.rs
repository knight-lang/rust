use crate::{Ast, Text, Variable, Custom, Null, Boolean, Number};
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) const SHIFT: usize = 3;
const MASK: u64 = (1 << SHIFT) as u64 - 1;

/// The value type within Knight.
///
/// You can interact with it with its associated functions.
pub struct Value(u64);

/// A trait that represents the ability for something to be a [`Value`].
///
/// # Safety
/// Implementors of this trait must guarantee that calling `is_value_a` on a [`Value`] will only ever return `true` if
/// the given `value` was constructed via `<Self as Into<Value>>::into()`, and must not return `true` for any other
/// type.
pub unsafe trait ValueKind<'a> : Debug + Clone + Into<Value> {
	/// The reference kind when [`downcast`](ValueKind::downcast_unchecked)ing a [`Value`].
	type Ref: Borrow<Self>;

	/// Checks to see if `value` is a `Self`.
	fn is_value_a(value: &Value) -> bool;

	/// Downcast the `value` to a [`Self::Ref`] without checking to see if `value` is a `Self`.
	unsafe fn downcast_unchecked(value: &'a Value) -> Self::Ref;

	/// Executes `self`.
	fn run(&self) -> crate::Result<Value>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tag {
	Constant = 0b000,
	Number   = 0b001,
	Variable = 0b010,
	Text     = 0b011,
	Ast      = 0b100,
	Custom   = 0b101,
}

impl Clone for Value {
	#[inline]
	fn clone(&self) -> Value {
		if !self.is_copy() {
			// SAFETY: All of `Variable`, `Text`, `Ast`, and `Custom`s have their first field as `AtomicUsize`s.
			unsafe {
				self.refcount().fetch_add(1, Ordering::Relaxed);
			}
		}

		// SAFETY: we just cloned all refcounted values, so we will have no double free errors.
		unsafe {
			self.copy()
		}
	}
}

impl Drop for Value {
	#[inline]
	fn drop(&mut self) {
		// we never want to inline this, as it's expensive to compute and not often called.
		#[cold]
		unsafe fn drop_inner(ptr: *mut (), tag: Tag) {
			match tag {
				Tag::Variable => Variable::drop_in_place(ptr),
				Tag::Text => Text::drop_in_place(ptr),
				Tag::Ast => Ast::drop_in_place(ptr),
				Tag::Custom => Custom::drop_in_place(ptr),
				_ => unreachable!()
			}
		}

		if self.is_copy() {
			return;
		}

		unsafe {
			let rc = self.refcount().fetch_sub(1, Ordering::Relaxed);

			debug_assert_ne!(rc, 0);

			if rc == 1 {
				drop_inner(self.ptr() as _, self.tag())
			}
		}
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		unsafe {
			match self.tag() {
				Tag::Constant if self.is_null() => Debug::fmt(&Null, f),
				Tag::Constant => Debug::fmt(&Boolean::downcast_unchecked(self), f),
				Tag::Number => Debug::fmt(&Number::downcast_unchecked(self), f),
				Tag::Text => Debug::fmt(&*Text::downcast_unchecked(self), f),
				Tag::Ast => Debug::fmt(&*Ast::downcast_unchecked(self), f),
				Tag::Variable => Debug::fmt(&*Variable::downcast_unchecked(self), f),
				Tag::Custom => Debug::fmt(&*Custom::downcast_unchecked(self), f),
			}
		}
	}
}

impl Value {
	pub fn is_null(&self) -> bool {
		self.raw() == Self::from(crate::Null).raw()
	}
}

impl Value {
	// pub fn new<K: for<'a> ValueKind<'a>>(kind: K) -> Self {
	// 	kind.into_value()
	// }

	// # SAFETY: `raw` must be a valid representation of a value.
	#[inline]
	pub(crate) const unsafe fn from_raw(raw: u64) -> Self {
		Self(raw)
	}

	// # SAFETY: `raw` must be a valid representation of a `tag`, and musn't have its lower bits set.
	#[inline]
	pub(crate) const unsafe fn new_tagged(raw: u64, tag: Tag) -> Self {
		debug_assert_eq_const!(raw & MASK, 0);

		Self::from_raw(raw | tag as u64)
	}

	#[inline]
	pub(crate) const fn raw(&self) -> u64 {
		self.0
	}

	pub(crate) const fn tag(&self) -> Tag {
		match self.0 & MASK {
			0b000 => Tag::Constant,
			0b001 => Tag::Number,
			0b010 => Tag::Variable,
			0b011 => Tag::Text,
			0b100 => Tag::Ast,
			0b101 => Tag::Custom,
			_ => {
				#[allow(unconditional_panic)]
				let _: () = [();0][1];
				Tag::Custom
			}
		}
	}

	pub(crate) const fn unmask(&self) -> u64 {
		self.0 & !MASK
	}

	pub(crate) const fn ptr(&self) -> *const () {
		self.unmask() as _
	}

	#[inline]
	const fn is_copy(&self) -> bool {
		const_assert!((Tag::Constant as u64) <= 1);
		const_assert!((Tag::Number as u64) <= 1);
		const_assert!((Tag::Variable as u64) > 1);
		const_assert!((Tag::Text as u64) > 1);
		const_assert!((Tag::Ast as u64) > 1);
		const_assert!((Tag::Custom as u64) > 1);

		(self.tag() as u64) <= 1
	}

	// SAFETY: must be a constant or a number.
	#[inline]
	unsafe fn copy(&self) -> Self {
		Self(self.0)
	}

	unsafe fn refcount(&self) -> &AtomicUsize {
		debug_assert!(!self.is_copy());

		&*(self.ptr() as *const AtomicUsize)
	}
}
