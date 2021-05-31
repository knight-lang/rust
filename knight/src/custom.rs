use crate::value::{Value, ValueKind, Tag, Runnable};
use std::{borrow::Borrow, ops::Deref};

#[derive(Debug, Clone)]
pub struct Custom<'env> {
	x: &'env ()
}

impl Custom<'_> {
	pub(crate) unsafe fn drop_in_place(ptr: *mut ()) {
		let _ = ptr;
		todo!();
	}
}

impl<'env> From<Custom<'env>> for Value<'env> {
	#[inline]
	fn from(custom: Custom<'env>) -> Self {
		let _ = custom;
		todo!()
	}
}

pub struct CustomRef<'a, 'env> {
	_x: &'a &'env () 
}

impl<'a, 'env> Borrow<Custom<'env>> for CustomRef<'a, 'env> {
	fn borrow(&self) -> &Custom<'env> {
		todo!()
	}
}

impl<'env> Deref for CustomRef<'_, 'env> {
	type Target = Custom<'env>;

	fn deref(&self) -> &Self::Target {
		todo!()
	}
}


unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Custom<'env> {
	type Ref = CustomRef<'value, 'env>;

	fn is_value_a(value: &Value<'env>) -> bool {
		value.tag() == Tag::Custom
	}

	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		todo!();
		// Self::new_unchecked((value.raw() as NumberInner) >> SHIFT)
	}
}

impl<'env> Runnable<'env> for Custom<'env> {
	fn run(&self, env: &'env mut crate::Environment) -> crate::Result<Value<'env>> {
		let _ = env;

		todo!();
	}
}
