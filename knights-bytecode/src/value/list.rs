use std::slice::Iter;

use crate::options::Options;
use crate::value::{Boolean, Integer, KString, NamedType, ToBoolean, ToInteger, ToKString, Value};
use crate::{Environment, Result};

// todo: optimize
#[derive(Clone, Debug)] // TODO: DEBUG
pub struct List(Option<Box<[Value]>>);

pub trait ToList {
	fn to_list(&self, env: &mut Environment) -> Result<List>;
}

impl NamedType for List {
	#[inline]
	fn type_name(&self) -> &'static str {
		"List"
	}
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

pub struct ListRefIter<'a>(Iter<'a, Value>);
impl<'a> Iterator for ListRefIter<'a> {
	type Item = &'a Value;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl<'a> IntoIterator for &'a List {
	type Item = &'a Value;
	type IntoIter = ListRefIter<'a>;
	fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
		ListRefIter(self.0.as_ref().map(|x| x.iter()).unwrap_or_default())
	}
}

impl List {
	pub fn concat<'a>(
		&self,
		other_list: impl IntoIterator<Item = Value>,
		opts: &Options,
	) -> Result<Self> {
		let _ = (other_list, opts);
		todo!();
	}

	pub fn difference<'a>(
		&self,
		other_list: impl IntoIterator<Item = &'a Value>,
		opts: &Options,
	) -> Result<Self> {
		let _ = (other_list, opts);
		todo!();
	}
}
