use crate::value::{Value, ValueKind, Tag};
use std::{borrow::Borrow, ops::Deref};

#[derive(Debug, Clone)]
pub struct Custom {

}

impl Custom {
	pub(crate) unsafe fn drop_in_place(ptr: *mut ()) {
		let _ = ptr;
		todo!();
	}
}

impl From<Custom> for Value {
	#[inline]
	fn from(custom: Custom) -> Self {
		let _ = custom;
		todo!()
	}
}

pub struct CustomRef<'a> {
	_x: &'a () 
}

impl<'a> Borrow<Custom> for CustomRef<'a> {
	fn borrow(&self) -> &Custom {
		todo!()
	}
}

impl Deref for CustomRef<'_> {
	type Target = Custom;

	fn deref(&self) -> &Self::Target {
		todo!()
	}
}


unsafe impl<'a> ValueKind<'a> for Custom {
	type Ref = CustomRef<'a>;

	fn is_value_a(value: &Value) -> bool {
		value.tag() == Tag::Custom
	}

	unsafe fn downcast_unchecked(value: &'a Value) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		todo!();
		// Self::new_unchecked((value.raw() as NumberInner) >> SHIFT)
	}

	fn run(&self) -> crate::Result<Value> {
		todo!();
	}
}