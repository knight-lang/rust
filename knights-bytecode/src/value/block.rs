use crate::old_vm_and_parser_and_program::program::JumpIndex;

use super::NamedType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block(JumpIndex);

impl NamedType for Block {
	fn type_name(&self) -> &'static str {
		"Block"
	}
}

impl Block {
	pub fn new(idx: JumpIndex) -> Self {
		Self(idx)
	}

	pub fn inner(self) -> JumpIndex {
		self.0
	}
}
