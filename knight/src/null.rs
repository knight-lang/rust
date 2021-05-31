use crate::{Value, Boolean, Number, Text};
use crate::value::{Tag, ValueKind, SHIFT, Runnable};
use std::fmt::{self, Display, Formatter};

/// The null type within Knight.
///
/// This notably doesn't implement [`Ord`]/[`PartialOrd`], as the Knight spec says that nulls cannot be compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Null;

impl Display for Null {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt("null", f)
	}
}

impl From<Null> for Value<'_> {
	#[inline]
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Null {
	type Ref = Self;

	fn is_value_a(value: &Value<'env>) -> bool {
		value.raw() == Value::NULL.raw()
	}

	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));
		let _ = value;

		Self
	}
}

impl<'env> Runnable<'env> for Null {
	fn run(&self, _: &'env  crate::Environment) -> crate::Result<Value<'env>> {
		Ok((*self).into())
	}
}

impl From<Null> for Number {
	#[inline]
	fn from(null: Null) -> Self {
		let _ = null;

		Self::ZERO
	}
}

impl Value<'_> {
	// note that 0 is false and 2 is true.
	pub const NULL: Self = unsafe { Value::new_tagged(1 << SHIFT, Tag::Constant) };
}


impl From<Null> for Boolean {
	#[inline]
	fn from(_: Null) -> Self {
		false
	}
}

impl From<Null> for Text {
	#[inline]
	fn from(_: Null) -> Self {
		// todo: use a static one
		unsafe {
			Self::new_unchecked(std::borrow::Cow::Borrowed("null"))
		}
	}
}