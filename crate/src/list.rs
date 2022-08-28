use crate::{RefCount, SharedText, Text, Value};
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;

#[derive(Default, Clone, PartialEq)]
pub struct List(RefCount<Vec<Value>>);

impl Debug for List {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_list().entries(&*self.as_slice()).finish()
	}
}

impl Deref for List {
	type Target = [Value];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl List {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn as_slice(&self) -> &[Value] {
		self
	}

	pub fn contains(&self, value: &Value) -> bool {
		self.as_slice().contains(value)
	}

	pub fn to_text(&self) -> crate::Result<SharedText> {
		const SEPARATOR: &Text = unsafe { Text::new_unchecked("\n") };

		let mut text = SharedText::builder();

		let mut first = true;
		for ele in self {
			if first {
				first = false;
			} else {
				text.push(SEPARATOR);
			}

			text.push(&ele.to_text()?);
		}

		Ok(text.finish())
	}

	pub fn concat(&self, rhs: &[Value]) -> Self {
		self.iter().cloned().chain(rhs.iter().cloned()).collect()
	}

	pub fn difference(&self, rhs: &[Value]) -> Self {
		let mut list = Vec::with_capacity(self.len() - rhs.len());

		for ele in self {
			if !rhs.contains(ele) && !list.contains(ele) {
				list.push(ele.clone());
			}
		}

		list.into()
	}

	pub fn repeat(&self, amount: usize) -> Self {
		let mut list = Vec::with_capacity(self.len() * amount);

		for _ in 0..amount {
			list.extend(self.iter().cloned());
		}

		list.into()
	}

	pub fn join(&self, sep: &Text) -> crate::Result<SharedText> {
		let mut joined = SharedText::builder();

		let mut is_first = true;
		for ele in self {
			if is_first {
				is_first = false;
			} else {
				joined.push(&sep);
			}

			joined.push(&ele.to_text()?);
		}

		Ok(joined.finish())
	}
}

impl From<Vec<Value>> for List {
	fn from(vec: Vec<Value>) -> Self {
		Self(vec.into())
	}
}

impl FromIterator<Value> for List {
	fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
		iter.into_iter().collect::<Vec<Value>>().into()
	}
}

impl<'a> IntoIterator for &'a List {
	type Item = &'a Value;
	type IntoIter = <&'a [Value] as IntoIterator>::IntoIter;

	fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
		self.0.iter()
	}
}

pub struct Iter<'a>(usize, &'a [Value]);

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
