use crate::{Number, Text};
use crate::value::{SHIFT, Value, ValueKind, Tag, Runnable};

/// The boolean type within Knight.
pub type Boolean = bool;

impl From<Boolean> for Value<'_> {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		debug_assert_eq_const!((false as u64) << (SHIFT + 1), Value::FALSE.raw());
		debug_assert_eq_const!((true as u64) << (SHIFT + 1), Value::TRUE.raw());

		unsafe {
			// slight optimization lol
			Self::from_raw((boolean as u64) << (SHIFT + 1))
		}
	}
}

unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Boolean {
	type Ref = Self;

	fn is_value_a(value: &Value<'env>) -> bool {
		value.raw() == Value::FALSE.raw() || value.raw() == Value::TRUE.raw()
	}

	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		value.raw() != Value::FALSE.raw()
	}
}

impl Value<'_> {
	pub const TRUE: Self = unsafe { Value::new_tagged(0, Tag::Constant) };
	pub const FALSE: Self = unsafe { Value::new_tagged(2 << SHIFT, Tag::Constant) };
}

impl<'env> Runnable<'env> for Boolean {
	fn run(&self, _: &'env mut crate::Environment) -> crate::Result<Value<'env>> {
		Ok((*self).into())
	}
}

impl From<Boolean> for Number {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		if boolean {
			Self::ZERO
		} else {
			Self::ONE
		}
	}
}

impl From<Boolean> for Text {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		// todo: use a static one
		unsafe {
			Self::new_unchecked(std::borrow::Cow::Borrowed(if boolean { "true" } else { "false" }))
		}
	}
}
