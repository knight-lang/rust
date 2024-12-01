mod compiler;

use crate::parser::{ParseErrorKind, SourceLocation, VariableName};
use crate::strings::StringSlice;
use crate::value::{KString, Value};
use crate::vm::Opcode;
use crate::Options;
pub use compiler::{Compilable, Compiler};
use std::fmt::{self, Debug, Formatter};

// todo: u32 vs u64? i did u64 bx `0x00ff_ffff` isn't a lot of offsets.
type InstructionAndOffset = u64;

/// A Program represents an executable Knight program.
///
/// After being parsed, Knight programs become [`Program`]s, which can then be run by
/// [`Vm`](crate::VM)s later on.
pub struct Program {
	// The code for the program. The bottom-most byte is the opcode, and when that's shifted away,
	// the remainder is the offset.
	code: Box<[InstructionAndOffset]>,

	// All the constants that've been seen in the program. Used by [`Opcode::PushConstant`].
	constants: Box<[Value]>,

	// The amount of variables in the program. Note that when `debug_assertions` are enabled,
	// `variable_names` also exists (as it's used for Debug formatting)
	num_variables: usize,

	// Only enabled when stacktrace printing is enabled, this is a map from the bytecode offset (ie
	// the index into `code`) to a source location. Only the first bytecode from each line is added
	// (to improve efficiency), so when looking up in `source_lines`, if a value doesn't exist you
	// need to iterate backwards until you find one.
	#[cfg(feature = "stacktrace")]
	source_lines: std::collections::HashMap<usize, SourceLocation>,

	// Only enabled when stacktrace printing is enabled, this is a mapping of jump indices (which
	// correspond to the first instruction of a [`Block`]) to the (optional) name of the block, and
	// the location where the block was declared.
	#[cfg(feature = "stacktrace")]
	// (IMPL NOTE: Technically, do we need the source location? it's not currently used in msgs.)
	block_locations: std::collections::HashMap<JumpIndex, (Option<VariableName>, SourceLocation)>,

	// The list of variable names. Only enabled when `debug_assertions` are on, as it's used within
	// the `Debug` implementation of `Program`.
	#[cfg(debug_assertions)]
	variable_names: Vec<VariableName>,
}

impl Debug for Program {
	/// Write the debug output for `Program`.
	///
	/// This also decodes the bytecode contained within the [`Program`], to make it easy understand
	/// what's happening.
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		struct Bytecode<'a>(&'a [InstructionAndOffset]);
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
		block: crate::value::Block,
	) -> Option<(Option<&VariableName>, &SourceLocation)> {
		self.block_locations.get(&block.inner()).map(|&(ref a, ref b)| (a.as_ref(), b))
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
