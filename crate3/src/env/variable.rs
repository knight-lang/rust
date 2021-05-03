use crate::{Value, Runnable, Environment, Result, Error};
use std::cell::RefCell;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Variable<'env>(*const VariableInner<'env>);

struct VariableInner<'env> {
	name: String,
	value: RefCell<Option<Value<'env>>>
}

impl<'env> Variable<'env> {
	fn inner(&self) -> &VariableInner<'env> {
		unsafe {
			&*self.0
		}
	}

	pub(crate) fn into_raw(self) -> *mut () {
		self.0 as *mut _
	}

	pub(crate) unsafe fn from_raw(raw: *mut ()) -> Self {
		Self(raw as *mut _)
	}

	pub fn name(&self) -> &str {
		&self.inner().name
	}

	pub fn value(&self) -> Option<Value<'env>> {
		self.inner().value.borrow().clone()
	}

	pub fn set_value(&self, value: Value<'env>) {
		*self.inner().value.borrow_mut() = Some(value);
	}
}

impl<'env> Runnable<'env> for Variable<'env> {
	fn run(&self, _: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
		self.value().ok_or_else(|| Error::UndefinedVariable(self.name().to_string()))
	}
}

impl Debug for Variable<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Variable")
				.field("name", &self.name())
				.field("value", &self.inner().value)
				.finish()
		} else {
			f.debug_tuple("Variable")
				.field(&self.name())
				.finish()
		}
	}
}