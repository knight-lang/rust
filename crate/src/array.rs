use crate::{Mutable, RefCount, Value};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Array(RefCount<Mutable<Vec<Value>>>);

impl Array {
	pub fn is_empty(&self) -> bool {
		self.0.borrow().is_empty()
	}
}
