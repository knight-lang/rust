use crate::program::JumpIndex;

use super::NamedType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block(JumpIndex);

impl NamedType for Block {
	fn type_name(&self) -> &'static str {
		"Block"
	}
}

impl Block {
	pub const fn new(idx: JumpIndex) -> Self {
		Self(idx)
	}

	pub fn inner(self) -> JumpIndex {
		self.0
	}
}
