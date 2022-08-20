use crate::{KnStr, Mutable, RefCount, SharedStr, Value};
use std::ops::{Deref, DerefMut};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Array(RefCount<Mutable<Vec<Value>>>);

impl Array {
	pub fn is_empty(&self) -> bool {
		self.0.borrow().is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.borrow().len()
	}

	pub fn as_slice(&self) -> AsSlice<'_> {
		AsSlice(self.0.borrow())
	}

	pub fn iter(&self) -> Iter<'_> {
		Iter(0, self.as_slice())
	}

	pub fn contains(&self, value: &Value) -> bool {
		self.as_slice().contains(value)
	}

	pub fn to_knstr(&self) -> SharedStr {
		// let mut s = String::new();
		todo!()
		// for c
	}
}

impl From<Vec<Value>> for Array {
	fn from(vec: Vec<Value>) -> Self {
		Self(RefCount::new(vec.into()))
	}
}

impl FromIterator<Value> for Array {
	fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
		iter.into_iter().collect::<Vec<Value>>().into()
	}
}

pub struct AsSlice<'a>(std::cell::Ref<'a, Vec<Value>>);
impl Deref for AsSlice<'_> {
	type Target = [Value];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub struct Iter<'a>(usize, AsSlice<'a>);

impl<'a> Iterator for Iter<'a> {
	type Item = Value;

	fn next(&mut self) -> Option<Self::Item> {
		let ret = self.1.get(self.0).cloned();

		if ret.is_some() {
			self.0 += 1;
		}

		ret
	}
}
