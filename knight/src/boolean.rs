use crate::value::{SHIFT, Value, ValueKind, Tag};

/// The boolean type within Knight.
pub type Boolean = bool;

const FALSE_VALUE: Value = unsafe { Value::new_tagged(0, Tag::Constant) };
const TRUE_VALUE: Value = unsafe { Value::new_tagged(2 << SHIFT, Tag::Constant) };

impl From<Boolean> for Value {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		debug_assert_eq_const!((false as u64) << (SHIFT + 1), FALSE_VALUE.raw());
		debug_assert_eq_const!((true as u64) << (SHIFT + 1), TRUE_VALUE.raw());

		unsafe {
			// slight optimization lol
			Self::from_raw((boolean as u64) << (SHIFT + 1))
		}
	}
}

unsafe impl<'a> ValueKind<'a> for Boolean {
	type Ref = Self;

	fn is_value_a(value: &Value) -> bool {
		value.raw() == FALSE_VALUE.raw() || value.raw() == TRUE_VALUE.raw()
	}

	unsafe fn downcast_unchecked(value: &'a Value) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		value.raw() != FALSE_VALUE.raw()
	}

	fn run(&self) -> crate::Result<Value> {
		Ok((*self).into())
	}
}
