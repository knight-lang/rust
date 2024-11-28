use crate::value::{Boolean, Integer, KString, ToBoolean, ToInteger, ToKString, Value};
use crate::{Environment, Result};

// todo: optimize
#[derive(Clone, Debug)] // TODO: DEBUG
pub struct List(Option<Box<[Value]>>);

pub trait ToList {
	fn to_list(&self, env: &mut Environment) -> Result<List>;
}

impl List {
	pub fn boxed(value: Value) -> Self {
		Self(Some(vec![value].into()))
	}
}

impl Default for List {
	fn default() -> Self {
		Self(None)
	}
}

impl ToBoolean for List {
	fn to_boolean(&self, env: &mut Environment) -> Result<Boolean> {
		todo!()
	}
}

impl ToKString for List {
	fn to_kstring(&self, env: &mut Environment) -> Result<KString> {
		todo!()
	}
}

impl ToInteger for List {
	fn to_integer(&self, env: &mut Environment) -> Result<Integer> {
		todo!()
	}
}

impl ToList for List {
	fn to_list(&self, _: &mut Environment) -> Result<List> {
		Ok(self.clone())
	}
}
