use crate::value::{Value, Tag, ValueKind};
use std::sync::Arc;
use std::{borrow::Borrow, ops::Deref};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ast(Arc<str>); // todo

impl Ast {
	fn into_raw(self) -> *mut () {
		Arc::into_raw(self.0) as _
	}

	pub(crate) unsafe fn drop_in_place(ptr: *mut ()) {
		let _ = ptr;
		todo!();
	}
}

impl From<Ast> for Value {
	fn from(text: Ast) -> Self {
		unsafe {
			Self::new_tagged(text.into_raw() as _, Tag::Ast)
		}
	}
}

pub struct AstRef<'a> {
	_x: &'a () 
}

impl Borrow<Ast> for AstRef<'_> {
	fn borrow(&self) -> &Ast {
		todo!()
	}
}

impl Deref for AstRef<'_> {
	type Target = Ast;

	fn deref(&self) -> &Self::Target {
		todo!()
	}
}

unsafe impl<'a> ValueKind<'a> for Ast {
	type Ref = AstRef<'a>;

	fn is_value_a(value: &Value) -> bool {
		value.tag() == Tag::Ast
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