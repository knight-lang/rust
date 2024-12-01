mod builder;

pub use builder::Builder;

use crate::options::Options;
use crate::value::KString;
use crate::vm::{Opcode, ParseErrorKind, SourceLocation};
use crate::{strings::StringSlice, Value};
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

pub struct Program {
	code: Box<[u64]>, // todo: u32 vs u64? i did u64 bx `0x00ff_ffff` isn't a lot of offsets.
	constants: Box<[Value]>,
	num_variables: usize,

	#[cfg(feature = "knight-debugging")]
	source_lines: HashMap<usize, SourceLocation>,

	#[cfg(feature = "knight-debugging")]
	functions: HashMap<JumpIndex, (Option<KString>, SourceLocation)>,

	#[cfg(debug_assertions)]
	variable_names: Vec<Box<StringSlice>>, // since it's only needed for debugging knightrs itself
}

impl Debug for Program {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		struct Bytecode<'a>(&'a [u64]);
		impl Debug for Bytecode<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				if !f.alternate() {
					return f.write_str("[...]");
				}

				let mut bytecode = f.debug_list();
				for (idx, &number) in self.0.into_iter().enumerate() {
					let opcode = unsafe { Opcode::from_byte_unchecked((number as u8)) };
					let offset = (number >> 0o10) as usize;
					if opcode.takes_offset() {
						bytecode.entry(&format!("{}: {:?} (offset={})", idx, opcode, offset));
					} else {
						bytecode.entry(&format!("{}: {:?}", idx, opcode));
					}
				}
				bytecode.finish()
			}
		}

		let mut prog = f.debug_struct("Program");
		prog.field("num_variables", &self.num_variables);
		prog.field("constants", &self.constants);
		prog.field("bytecode", &Bytecode(&self.code));

		#[cfg(debug_assertions)]
		prog.field("variables", &self.variable_names);

		prog.finish()
	}
}

impl Program {
	// SAFETY: `offset` needs to be <= the code length.
	pub unsafe fn opcode_at(&self, offset: usize) -> (Opcode, usize) {
		debug_assert!(offset < self.code.len());
		// SAFETY: caller ensures offset is correct
		let number = unsafe { *self.code.get_unchecked(offset) };

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

	#[cfg(feature = "knight-debugging")]
	pub fn source_location_at(offset: usize) -> SourceLocation {
		Default::default()
	}

	#[cfg(feature = "knight-debugging")]
	pub fn function_name(
		&self,
		index: JumpIndex,
	) -> Option<(Option<&StringSlice>, &SourceLocation)> {
		self.functions.get(&index).map(|(idx, loc)| (idx.as_deref(), loc))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JumpIndex(pub(super) usize);

#[derive(Debug, PartialEq, Eq)]
pub struct DeferredJump(usize, JumpWhen);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpWhen {
	True,
	False,
	Always,
}
