use crate::value::{Integer, List, ToInteger, ToList};
use crate::Result;

pub type Boolean = bool;

pub trait ToBoolean {
	fn to_boolean(&self) -> Result<Boolean>;
}

impl ToBoolean for Boolean {
	fn to_boolean(&self) -> Result<Self> {
		Ok(*self)
	}
}

impl ToInteger for Boolean {
	fn to_integer(&self) -> Result<Integer> {
		if *self {
			Ok(Integer::ONE)
		} else {
			Ok(Integer::ZERO)
		}
	}
}

impl ToList for Boolean {
	fn to_list(&self) -> Result<List> {
		if *self {
			Ok(List::boxed((*self).into()))
		} else {
			Ok(List::EMPTY)
		}
	}
}
