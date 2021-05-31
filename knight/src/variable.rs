use std::fmt::{self, Debug, Formatter};
use crate::value::{Value, Tag, ValueKind};
use std::cell::RefCell;
use std::{borrow::Borrow, ops::Deref};

/// A Variable within Knight, which can be used to store values.
///
/// Variables are considered the same if they're identical.
#[derive(Clone, Copy, PartialEq, Eq)] // you can copy variables as theyre just references. The environment drops the `VariableInner` for us.
#[repr(transparent)]
pub struct Variable<'env>(*const VariableInner<'env>);

struct VariableInner<'env> {
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

impl<'env> Variable<'env> {
	pub(crate) fn new(name: Box<str>) -> Self {
		let inner = Box::new(VariableInner { name, value: RefCell::new(None) });

		Self(Box::leak(inner) as *const VariableInner<'env>)
	}

	fn into_raw(self) -> *const () {
		self.0 as _
	}

	// SAFETY: `raw` must have been returned from `into_raw`
	#[allow(unused)]
	unsafe fn from_raw(raw: *const ()) -> Self {
		Self(raw as *const VariableInner<'env>)
	}

	fn inner(self) -> &'env VariableInner<'env> {
		unsafe { &*self.0 }
	}

	/// Gets the name associated with this variable.
	pub fn name(self) -> &'env str {
		&self.inner().name
	}

	/// Fetches the variable associated with this variable, returning [`None`] if it was never assigned.
	pub fn get(self) -> Option<Value<'env>> {
		self.inner().value.borrow().clone()
	}

	/// Associates `value` with `self`, so the next time [`Self::get`] is called, it will be referenced.
	pub fn set(self, value: Value<'env>) {
		*self.inner().value.borrow_mut() = Some(value);
	}

	pub(crate) unsafe fn drop_in_place(self) {
		(self.0 as *mut VariableInner<'env>).drop_in_place();
	}
}

impl<'env> From<Variable<'env>> for Value<'env> {
	fn from(var: Variable<'env>) -> Self {
		unsafe {
			Self::new_tagged(var.into_raw() as _, Tag::Variable)
		}
	}
}

#[repr(transparent)]
pub struct VariableRef<'a, 'env: 'a>(&'a VariableInner<'env>);

impl<'env> Borrow<Variable<'env>> for VariableRef<'_, 'env> {
	fn borrow(&self) -> &Variable<'env> {
		&self
	}
}

impl<'env> Deref for VariableRef<'_, 'env> {
	type Target = Variable<'env>;

	fn deref(&self) -> &Self::Target {
		// SAFETY:
		// `Variable` is a transparent pointer to `VariableInner` whereas `VariableRef` is a transparent
		// reference to the same type. Since pointers and references can be transmuted safely, this is valid.
		unsafe {
			std::mem::transmute::<&VariableRef<'_, 'env>, &Variable<'env>>(self)
		}
	}
}

unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Variable<'env> {
	type Ref = VariableRef<'value, 'env>;

	fn is_value_a(value: &Value<'env>) -> bool {
		value.tag() == Tag::Variable
	}

	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		VariableRef(&*(value.ptr() as *const VariableInner<'env>))
	}

	fn run(&self, _: &'env mut crate::Environment) -> crate::Result<Value<'env>> {
		self.get().ok_or_else(|| crate::Error::UndefinedVariable(self.inner().name.clone()))
	}
}
