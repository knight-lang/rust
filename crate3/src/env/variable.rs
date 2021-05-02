use crate::{Value, ValueType, Environment, Result, Error};
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

	pub(crate) fn into_raw(self) -> *const () {
		self.0 as *const _
	}

	pub(crate) unsafe fn from_raw(raw: *const ()) -> Self {
		Self(raw as *const _)
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

impl<'env> ValueType<'env> for Variable<'env> {
	fn run(&self, env: &'env mut Environment) -> Result<Value<'env>> {
		self.value().ok_or_else(|| Error::UndefinedVariable(self.name().to_string()))
	}
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