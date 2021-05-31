use crate::{Ast, Text, Variable, Custom, Null, Boolean, Number, Environment, Error};
use std::borrow::Borrow;
use crate::ops::Runnable;
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::marker::PhantomData;
use std::convert::TryFrom;

pub(crate) const SHIFT: usize = 3;
const MASK: u8 = (1u8 << SHIFT) - 1;

/// The value type within Knight.
///
/// Since [`Value`]s can be [`Variable`]s, they must be associated with a given environment, hence the lifetime.
#[repr(transparent)]
pub struct Value<'env>(u64, PhantomData<&'env ()>);

/// A trait that represents the ability for something to be a [`Value`].
///
/// # Safety
/// Implementors of this trait must guarantee that calling `is_value_a` on a [`Value`] will only ever return `true` if
/// the given `value` was constructed via `<Self as Into<Value>>::into()`, and must not return `true` for any other
/// type.
pub unsafe trait ValueKind<'value, 'env: 'value> : Debug + Clone + Into<Value<'env>> + Runnable<'env> {
	/// The reference kind when [`downcast`](ValueKind::downcast_unchecked)ing a [`Value`].
	type Ref: Borrow<Self>;

	/// Checks to see if `value` is a `Self`.
	fn is_value_a(value: &Value<'env>) -> bool;

	/// Downcast the `value` to a [`Self::Ref`] without checking to see if `value` is a `Self`.
	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Tag {
	Constant = 0b000,
	Number   = 0b001,
	Variable = 0b010,
	Text     = 0b011,
	Ast      = 0b100,
	Custom   = 0b101,
}

impl Clone for Value<'_> {
	#[inline]
	fn clone(&self) -> Self {
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

impl Drop for Value<'_> {
	#[inline]
	fn drop(&mut self) {
		// we never want to inline this, as it's expensive to compute and not often called.
		#[cold]
		unsafe fn drop_inner(ptr: *mut (), tag: Tag) {
			match tag {
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

impl Debug for Value<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		unsafe {
			match self.tag() {
				Tag::Constant if Null::is_value_a(self) => Debug::fmt(&Null, f),
				Tag::Constant => Debug::fmt(&Boolean::downcast_unchecked(self), f),
				Tag::Number => Debug::fmt(&Number::downcast_unchecked(self), f),
				Tag::Variable => Debug::fmt(&Variable::downcast_unchecked(self), f),
				Tag::Text => Debug::fmt(&*Text::downcast_unchecked(self), f),
				Tag::Ast => Debug::fmt(&*Ast::downcast_unchecked(self), f),
				Tag::Custom => Debug::fmt(&*Custom::downcast_unchecked(self), f),
			}
		}
	}
}

impl Default for Value<'_> {
	fn default() -> Self {
		Self::NULL
	}
}

impl<'env> Runnable<'env> for Value<'env> {
	fn run(&self, env: &'env  Environment) -> crate::Result<Value<'env>> {
		// in order of liklihood.
		if let Some(ast) = self.downcast::<Ast>() {
			ast.run(env)
		} else if let Some(variable) = self.downcast::<Variable>() {
			variable.run(env)
		} else if let Some(text) = self.downcast::<Text>() {
			text.run(env)
		} else {
			debug_assert!(matches!(self.tag(), Tag::Constant | Tag::Number));
			unsafe {
				Ok(self.copy())
			}
		}
	}
}

impl<'env> Value<'env> {
	// pub fn new<K: for<'a> ValueKind<'a>>(kind: K) -> Self {
	// 	kind.into_value()
	// }

	pub fn is_a<'a, T: ValueKind<'a, 'env>>(&'a self) -> bool {
		T::is_value_a(self)
	}

	pub unsafe fn downcast_unchecked<'a, T: ValueKind<'a, 'env>>(&'a self) -> T::Ref {
		debug_assert!(self.is_a::<T>());

		T::downcast_unchecked(self)
	}

	pub fn downcast<'a, T: ValueKind<'a, 'env>>(&'a self) -> Option<T::Ref> {
		if self.is_a::<T>() {
			unsafe {
				Some(self.downcast_unchecked::<T>())
			}
		} else {
			None
		}
	}

	// # SAFETY: `raw` must be a valid representation of a value.
	#[inline]
	pub(crate) const unsafe fn from_raw(raw: u64) -> Self {
		Self(raw, PhantomData)
	}

	// # SAFETY: `raw` must be a valid representation of a `tag`, and musn't have its lower bits set.
	#[inline]
	pub(crate) const unsafe fn new_tagged(raw: u64, tag: Tag) -> Self {
		debug_assert_eq_const!(raw & (MASK as u64), 0);

		Self::from_raw(raw | tag as u64)
	}

	#[inline(always)]
	pub(crate) const fn raw(&self) -> u64 {
		self.0
	}

	pub(crate) fn tag(&self) -> Tag {
		let raw_tag = (self.0 as u8) & MASK;

		debug_assert!(raw_tag <= Tag::Custom as u8);

		unsafe {
			std::mem::transmute::<u8, Tag>(raw_tag)
		}
	}

	pub(crate) const fn unmask(&self) -> u64 {
		self.0 & !(MASK as u64)
	}

	pub(crate) const fn ptr(&self) -> *const () {
		self.unmask() as _
	}

	#[inline]
	fn is_copy(&self) -> bool {
		const_assert!((Tag::Constant as u64) <= 2);
		const_assert!((Tag::Number as u64) <= 2);
		const_assert!((Tag::Variable as u64) <= 2);
		const_assert!((Tag::Text as u64) > 2);
		const_assert!((Tag::Ast as u64) > 2);
		const_assert!((Tag::Custom as u64) > 2);

		(self.tag() as u64) <= 2
	}

	#[inline]
	fn is_literal(&self) -> bool {
		(self.tag() as u64) <= 1
	}

	// checks to see if self is a literal _and_ falsey
	pub(crate) fn is_falsey_literal(&self) -> bool {
		self.raw() <= Self::NULL.raw()
	}

	// SAFETY: must be a constant or a number.
	#[inline]
	unsafe fn copy(&self) -> Self {
		Self(self.0, PhantomData)
	}

	unsafe fn refcount(&self) -> &AtomicUsize {
		debug_assert!(!self.is_copy());

		&*(self.ptr() as *const AtomicUsize)
	}

	pub fn typename(&self) -> &'static str {
		match self.tag() {
			Tag::Constant if Null::is_value_a(self) => "Null",
			Tag::Constant => "Boolean",
			Tag::Number => "Number",
			Tag::Variable => "Variable",
			Tag::Text => "Text",
			Tag::Ast => "Ast",
			Tag::Custom => "Custom",
		}
	}
}

// n.b.: while we could use a `match` in these cases, we order them in liklihood.
impl<'env> Value<'env> {
	pub fn to_number(&self) -> crate::Result<Number> {
		if let Some(number) = self.downcast::<Number>() {
			Ok(number)
		} else if let Some(textref) = self.downcast::<Text>() {
			Ok(Number::try_from(textref)?.into())
		} else if self.is_literal() {
			Ok(if self.raw() == Self::TRUE.raw() { Number::ONE } else { Number::ZERO })
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), into: "Number" })
		}
	}

	pub fn to_text(&self) -> crate::Result<Text> {
		if let Some(text) = self.downcast::<Text>() {
			Ok((*text).clone())
		} else if let Some(number) = self.downcast::<Number>() {
			Ok(number.into())
		} else if let Some(boolean) = self.downcast::<Boolean>() {
			Ok(boolean.into())
		} else if let Some(null) = self.downcast::<Null>() {
			Ok(null.into())
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), into: "Number" })
		}
	}

	pub fn to_boolean(&self) -> crate::Result<Boolean> {

		if self.is_literal() {
			let is_true = self.raw() <= Self::NULL.raw();

			debug_assert_eq!(
				is_true,
				self.raw() == Self::NULL.raw()
					|| self.raw() == Self::FALSE.raw()
					|| self.raw() == unsafe { Self::new_tagged(0, Tag::Number).raw() }
			);

			return Ok(is_true);
		}

		if self.is_a::<Text>() {
			unsafe {
				Ok(!self.downcast_unchecked::<Text>().is_empty())
			}
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), into: "Boolean" })
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_literal() {
		assert!(Value::TRUE.is_literal());
		assert!(Value::FALSE.is_literal());
		assert!(Value::NULL.is_literal());
		assert!(Value::from(Number::new(0).unwrap()).is_literal());
		assert!(Value::from(Number::new(1).unwrap()).is_literal());
		assert!(Value::from(Number::new(123).unwrap()).is_literal());
		assert!(Value::from(Number::new(-1).unwrap()).is_literal());

		assert!(!Value::from(Text::new("".into()).unwrap()).is_literal());
		assert!(!Value::from(Text::new("A".into()).unwrap()).is_literal());

		// todo: from value and ast.
	}
}