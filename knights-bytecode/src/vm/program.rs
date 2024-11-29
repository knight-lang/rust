use super::Opcode;
use crate::options::Options;
use crate::{strings::StringSlice, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Program {
	code: Box<[u64]>, // todo: u32 vs u64? i did u64 bx `0x00ff_ffff` isn't a lot of offsets.
	constants: Box<[Value]>,
	num_variables: usize,
}

impl Program {
	pub fn builder(opts: &Options) -> Builder<'_> {
		Builder::new(opts)
	}

	pub fn opcode_at(&self, offset: usize) -> (Opcode, usize) {
		let number = self.code[offset];

		// SAFETY: we know as this type was constructed that all programs result
		// in valid opcodes
		let opcode = unsafe { Opcode::from_byte_unchecked((number as u8)) };
		let offset = (number >> 0o10) as usize;

		(opcode, offset)
	}

	pub fn constant_at(&self, offset: usize) -> &Value {
		&self.constants[offset]
	}

	pub fn num_variables(&self) -> usize {
		self.num_variables
	}
}

pub struct Builder<'opts> {
	code: Vec<u64>,
	constants: Vec<Value>,
	variables: HashMap<Box<StringSlice>, usize>,
	opts: &'opts Options,
}

impl<'opts> Builder<'opts> {
	pub fn new(opts: &'opts Options) -> Self {
		Self { code: Vec::new(), constants: Vec::new(), variables: HashMap::new(), opts }
	}

	// needs to be called while ensuring soemthing's ont eh stack.
	pub unsafe fn build(mut self) -> Program {
		unsafe {
			self.opcode_without_offset(Opcode::Return);
		}

		Program {
			code: self.code.into(),
			constants: self.constants.into(),
			num_variables: self.variables.len(),
		}
	}

	// SAFETY: `opcode` must take an offset and `offset` must be a valid offset for it.
	unsafe fn opcode_with_offset(&mut self, opcode: Opcode, offset: usize) {
		// No need to check if `offset as u64`'s topbit is nonzero, as that's so massive it'll never happen
		self.code.push((opcode as u64 | (offset as u64) << 0o10));
	}

	// SAFETY: `opcode` mustn't take an offset
	unsafe fn opcode_without_offset(&mut self, opcode: Opcode) {
		self.code.push(opcode as u64);
	}

	pub fn push_constant(&mut self, value: Value) {
		let index = match self.constants.iter().enumerate().find(|(_, v)| value == **v) {
			Some((index, _)) => index,
			None => {
				let i = self.constants.len();
				self.constants.push(value);
				i
			}
		};

		// SAFETY: we know that `index` is a valid constant cause we just checked
		unsafe {
			self.opcode_with_offset(Opcode::PushConstant, index);
		}
	}

	// SAFETY: call ensures that the stack at any point when this opcode is run will have at least 2 values
	pub unsafe fn add(&mut self) {
		self.opcode_without_offset(Opcode::Add);
	}

	pub fn push_variable(&mut self, name: &StringSlice) {
		// TODO: check for name size (also in `set`)
		let index = match self.variables.get(name) {
			Some(&index) => index,
			None => {
				let i = self.variables.len();
				self.variables.insert(name.into_boxed(), i);
				i
			}
		};

		unsafe {
			self.opcode_with_offset(Opcode::GetVar, index);
		}
	}

	// SAFETY: when called, a value has to be on the stack
	pub unsafe fn set_variable(&mut self, name: &StringSlice) {
		let index = match self.variables.get(name) {
			Some(&index) => index,
			None => {
				let i = self.variables.len();
				self.variables.insert(name.into_boxed(), i);
				i
			}
		};

		unsafe {
			self.opcode_with_offset(Opcode::SetVar, index);
		}
	}

	pub fn prompt(&mut self) {
		unsafe {
			self.opcode_without_offset(Opcode::Prompt);
		}
	}

	pub fn random(&mut self) {
		unsafe {
			self.opcode_without_offset(Opcode::Random);
		}
	}
}
