use std::fmt::{self, Debug, Formatter};
use crate::value::{Value, Tag, ValueKind};
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{borrow::Borrow, ops::Deref};


pub struct Environment {

}

/// A Variable within Knight, which can be used to store values.
#[repr(transparent)]
pub struct Variable<'env>(*const VariableInner<'env>);

struct VariableInner<'env> {
	// 
	name: Box<str>,
	value: RefCell<Option<Value<'env>>>
}

impl Debug for Variable<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Variable")
				.field("name", &self.name())
				.field("value", &self.inner().value.borrow())
				.finish()
		} else {
			f.debug_tuple("Variable")
				.field(&self.name())
				.finish()
		}
	}
}

impl Clone for Variable<'_> {
	fn clone(&self) -> Self {
		self.inner().rc.fetch_add(1, Ordering::Relaxed);

		Self(self.0)
	}
}

impl Drop for Variable<'_> {
	fn drop(&mut self) {
		let rc = self.inner().rc.fetch_sub(1, Ordering::Relaxed);

		debug_assert_ne!(rc, 0);

		if rc == 1 {
			unsafe {
				Self::drop_in_place(self.0 as _);
			}
		}
	}
}

impl Variable {
	fn into_raw(self) -> *const () {
		self.0 as _
	}

	pub(crate) unsafe fn drop_in_place(ptr: *mut ()) {
		let ptr = ptr as *mut VariableInner;

		debug_assert_eq!((*ptr).rc.load(Ordering::Relaxed), 0);

		std::ptr::drop_in_place(ptr);
	}

	// SAFETY: `raw` must have been returned from `into_raw`
	#[allow(unused)]
	unsafe fn from_raw(raw: *const ()) -> Self {
		Self(raw as *const VariableInner)
	}

	fn inner(&self) -> &VariableInner {
		unsafe { &*self.0 }
	}

	/// Gets the name associated with this variable.
	pub fn name(&self) -> &str {
		&self.inner().name
	}

	/// Fetches the variable associated with this variable, returning [`None`] if it was never assigned.
	pub fn get(&self) -> Option<Value> {
		self.inner().value.borrow().clone()
	}

	/// Associates `value` with `self`, so the next time [`Self::get`] is called, it will be referenced.
	pub fn set(&self, value: Value) {
		*self.inner().value.borrow_mut() = Some(value);
	}
}

impl From<Variable> for Value {
	fn from(var: Variable) -> Self {
		unsafe {
			Self::new_tagged(var.into_raw() as _, Tag::Variable)
		}
	}
}

#[repr(transparent)]
pub struct VariableRef<'a>(&'a VariableInner);

impl<'a> Borrow<Variable> for VariableRef<'a> {
	fn borrow(&self) -> &Variable {
		&self
	}
}

impl Deref for VariableRef<'_> {
	type Target = Variable;

	fn deref(&self) -> &Self::Target {
		// SAFETY:
		// `Variable` is a transparent pointer to `VariableInner` whereas `VariableRef` is a transparent
		// reference to the same type. Since pointers and references can be transmuted safely, this is valid.
		unsafe {
			std::mem::transmute::<&VariableRef<'_>, &Variable>(self)
		}
	}
}

unsafe impl<'a> ValueKind<'a> for Variable {
	type Ref = VariableRef<'a>;

	fn is_value_a(value: &Value) -> bool {
		value.tag() == Tag::Variable
	}

	unsafe fn downcast_unchecked(value: &'a Value) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		VariableRef(&*(value.ptr() as *const VariableInner))
	}

	fn run(&self) -> crate::Result<Value> {
		self.get().ok_or_else(|| crate::Error::UndefinedVariable(self.inner().name.clone()))
	}
}