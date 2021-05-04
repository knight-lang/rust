use crate::{Value, Runnable, Environment, Result, Error};
use std::cell::RefCell;
use std::sync::Arc;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
#[repr(transparent)]
pub struct Variable(Arc<VariableInner>);

#[repr(C, align(8))] // rust?
struct VariableInner {
	name: String,
	value: RefCell<Option<Value>>
}

impl Variable {
	pub(super) fn create(name: String) -> Self {
		Self(Arc::new(VariableInner { name, value: RefCell::new(None) }))
	}

	fn inner(&self) -> &VariableInner {
		unsafe {
			&*self.0
		}
	}

	pub(crate) fn into_raw(self) -> *const () {
		Arc::into_raw(self.0) as _
	}

	pub(crate) unsafe fn from_raw(raw: *const ()) -> Self {
		Self(Arc::from_raw(raw as _))
	}

	pub fn name(&self) -> &str {
		&self.inner().name
	}

	pub fn value(&self) -> Option<Value> {
		self.inner().value.borrow().clone()
	}

	pub fn set_value(&self, value: Value) {
		*self.inner().value.borrow_mut() = Some(value);
	}
}

impl Runnable for Variable {
	fn run(&self, _: &mut Environment<'_, '_, '_>) -> Result<Value> {
		self.value().ok_or_else(|| Error::UndefinedVariable(self.name().to_string()))
	}
}

impl Debug for Variable {
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

impl Eq for Variable {}
impl PartialEq for Variable {
	fn eq(&self, rhs: &Self) -> bool {
		Arc::ptr_eq(&self.0, &rhs.0)
	}
}

impl std::borrow::Borrow<str> for Variable {
	fn borrow(&self) -> &str {
		self.name()
	}
}

impl std::hash::Hash for Variable {
	fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
		(&*self.0 as *const _ as usize).hash(h);
	}
}