use crate::value::{Boolean, Integer, List, ToBoolean, ToInteger, ToList};
use crate::Result;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null;

impl ToBoolean for Null {
	fn to_boolean(&self) -> Result<Boolean> {
		Ok(false)
	}
}

impl ToInteger for Null {
	fn to_integer(&self) -> Result<Integer> {
		Ok(Integer::ZERO)
	}
}

impl ToList for Null {
	fn to_list(&self) -> Result<List> {
		Ok(List::EMPTY)
	}
}
