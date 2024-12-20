mod compiler;

use crate::parser::{ParseErrorKind, SourceLocation, VariableName};
use crate::strings::KnStr;
use crate::value::{KnString, Value};
use crate::vm::{Callsite, Opcode};
use crate::Options;
pub use compiler::{Compilable, Compiler};
use indexmap::IndexSet;
use std::fmt::{self, Debug, Formatter};

// todo: u32 vs u64? i did u64 bx `0x00ff_ffff` isn't a lot of offsets.
type InstructionAndOffset = i64;

/// A Program represents an executable Knight program.
///
/// After being parsed, Knight programs become [`Program`]s, which can then be run by
/// [`Vm`](crate::VM)s later on.
pub struct Program<'src, 'path> {
	// The code for the program. The bottom-most byte is the opcode, and when that's shifted away,
	// the remainder is the offset.
	code: Box<[InstructionAndOffset]>,

	// All the constants that've been seen in the program. Used by [`Opcode::PushConstant`].
	constants: Box<[Value]>,

	// The list of variable names.
	variables: IndexSet<VariableName<'src>>,

	// Only enabled when stacktrace printing is enabled, this is a map from the bytecode offset (ie
	// the index into `code`) to a source location. Only the first bytecode from each line is added
	// (to improve efficiency), so when looking up in `source_lines`, if a value doesn't exist you
	// need to iterate backwards until you find one.
	#[cfg(feature = "stacktrace")]
	source_lines: std::collections::HashMap<usize, SourceLocation<'path>>,

	// Only enabled when stacktrace printing is enabled, this is a mapping of jump indices (which
	// correspond to the first instruction of a [`Block`]) to the (optional) name of the block, and
	// the location where the block was declared.
	#[cfg(feature = "stacktrace")]
	// (IMPL NOTE: Technically, do we need the source location? it's not currently used in msgs.)
	block_locations:
		std::collections::HashMap<JumpIndex, (Option<VariableName<'src>>, SourceLocation<'path>)>,

	// Needed for `'src` when qol and stacktrace aren't enabled.
	_ignored: (&'src (), &'path ()),
}

/// A type that represents a place programs can jump to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JumpIndex(pub(super) usize);

/// Represents a jump that's been deferred---it'll be reified once we know the target destination.
///
/// It's usually used when jumping forward to a location that's yet to be determined.
#[derive(Debug, PartialEq, Eq)] // Not `Clone` or `Copy` so we can't accidentally jump twice.
pub struct DeferredJump(usize, JumpWhen);

/// The condition for when to jump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpWhen {
	/// Jump only when the topmost element on the stack is truthy. (This'll pop the stack.)
	True,

	/// Jump only when the topmost element on the stack is falsey. (This'll pop the stack.)
	False,

	/// Always jump.
	Always,
}

impl Debug for Program<'_, '_> {
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
		prog.field("constants", &self.constants);
		prog.field("bytecode", &Bytecode(&self.code));
		prog.field("variables", &self.variables);

		prog.finish()
	}
}

impl<'src, 'path> Program<'src, 'path> {
	/// Gets the opcode, and its offset, at `offset`.
	///
	/// # Safety
	/// `location` must be `<` the source code's length.
	#[inline]
	pub unsafe fn opcode_at(&self, location: usize) -> (Opcode, usize) {
		debug_assert!(location < self.code.len());

		// SAFETY: caller ensures the locationis correct.
		let number = unsafe { *self.code.get_unchecked(location) };

		// SAFETY: we know as this type was constructed that all programs result
		// in valid opcodes
		let opcode = unsafe { Opcode::from_byte_unchecked((number as u8)) };
		let location = (number >> 0o10) as usize;

		(opcode, location)
	}

	/// Gets constant constant at `offset`.
	///
	/// # Safety
	/// `offset` must be a valid offset into the list of constants.
	pub unsafe fn constant_at(&self, offset: usize) -> &Value {
		debug_assert!(offset < self.constants.len());
		unsafe { self.constants.get_unchecked(offset) }
	}

	/// The number of variables that're defined in this program.
	#[inline]
	pub fn num_variables(&self) -> usize {
		self.variables.len()
	}

	/// Gets the variable at `idx`.
	pub fn variable_name(&self, var_idx: usize) -> &VariableName<'src> {
		&self.variables[var_idx]
	}

	/// Gets the variable at `idx`.
	#[cfg(feature = "extensions")]
	pub fn variable_index(&self, name: &VariableName<'_>) -> Option<usize> {
		self.variables.get_index_of(name)
	}

	/// Gets the source location at the program offset `offset`.
	///
	/// If `offset` doesn't directly map to a known source location, [`source_location_at`] works
	/// backwards until one is found. (Offset of `0` always has a source location.)
	#[cfg(feature = "stacktrace")]
	pub fn source_location_at(&self, mut offset: usize) -> SourceLocation<'path> {
		loop {
			// Note that this will never go below zero, as the first line is always recorded
			match self.source_lines.get(&offset) {
				Some(loc) => return loc.clone(),
				None => offset -= 1,
			}
		}
	}
}
