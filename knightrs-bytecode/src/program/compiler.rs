use super::{DeferredJump, InstructionAndOffset, JumpIndex, JumpWhen, Program};
use crate::gc::Gc;
use crate::options::Options;
use crate::parser::{ParseError, ParseErrorKind, SourceLocation, VariableName};
use crate::strings::KnStr;
use crate::value::Value;
use crate::vm::Opcode;

use indexmap::IndexSet;
use std::collections::HashMap;

// safety: cannot do invalid things with the builder.
pub unsafe trait Compilable<'src, 'path, 'gc> {
	// no errors returned because compiling should never fail, that's parsing
	fn compile(
		self,
		compiler: &mut Compiler<'src, 'path, 'gc>,
		opts: &Options,
	) -> Result<(), ParseError<'path>>;
}

/// A Compiler is used to construct [`Program`]s, which are then run via the [`Vm`](crate::Vm).
pub struct Compiler<'src, 'path, 'gc> {
	// The current code so far; The bottom-most byte is the opcode, and when that's shifted away, the
	// remainder is the offset.
	code: Vec<InstructionAndOffset>,

	gc: &'gc Gc,

	// All the constants that've been declared so far. Used with [`Opcode::PushConstant`].
	constants: Vec<Value<'gc>>,

	// The list of all variables encountered so far. (They're stored in an ordered set, as their
	// index is the "offset" that all `Opcodes` that interact with variables (eg [`Opcode::GetVar`])
	// will use.)
	variables: IndexSet<VariableName<'src>>,

	// Only enabled when stacktrace printing is enabled, this is a map from the bytecode offset (ie
	// the index into `code`) to a source location; Only the first bytecode from each line is added,
	// so when looking up in the `source_lines`, you need to
	#[cfg(feature = "stacktrace")]
	source_lines: HashMap<usize, SourceLocation<'path>>,

	// Only enabled when stacktrace printing is enabled, this is a mapping of jump indices (which
	// correspond to the first instruction of a [`Block`]) to the (optional) name of the block, and
	// the location where the block was declared.
	#[cfg(feature = "stacktrace")]
	block_locations: HashMap<JumpIndex, (Option<VariableName<'src>>, SourceLocation<'path>)>,

	// TODO: not public
	pub loops: Vec<(JumpIndex, Vec<DeferredJump>)>,

	// Needed for when `stacktrace` is disabled
	_ignored: &'path (),
}

fn code_from_opcode_and_offset(opcode: Opcode, offset: usize) -> InstructionAndOffset {
	opcode as InstructionAndOffset | (offset as InstructionAndOffset) << 0o10
}

// TODO: Make a "build-a-block" function
impl<'src, 'path, 'gc> Compiler<'src, 'path, 'gc> {
	#[cfg(feature = "extensions")]
	pub const ARGV_VARIABLE_INDEX: usize = 0;

	pub fn new(start: SourceLocation<'path>, gc: &'gc Gc) -> Self {
		Self {
			code: vec![],
			constants: vec![],
			gc,
			variables: {
				let mut variables = IndexSet::new();

				// Always add `_argv` in so that in `vm` we can always `set_variable` and not have UB
				// if the user didn't make  acompiler with argv
				#[cfg(feature = "extensions")]
				variables.insert(VariableName::new_unvalidated(&KnStr::new_unvalidated("_argv")));

				variables
			},

			#[cfg(feature = "stacktrace")]
			source_lines: {
				let mut sl = HashMap::new();
				sl.insert(0, start.clone());
				sl
			},

			#[cfg(feature = "stacktrace")]
			block_locations: {
				let mut bl = HashMap::new();
				bl.insert(JumpIndex(0), (None, start));
				bl
			},
			_ignored: &(),
			loops: vec![],
		}
	}
	/// Finished building the [`Program`], and returns it
	///
	/// # Safety
	/// The caller must ensure that the "program" that has been designed will have exactly one new
	/// value on top of its stack whenever it returns, which is the return value of the program.
	///
	/// Additionally, the caller must enure that all deferred jumps have been `jump_to`'d
	pub unsafe fn build(mut self) -> Program<'src, 'path, 'gc> {
		// SAFETY: The caller guarantees that we'll always have exactly one opcode on the top when
		// the program is finished executing, so we know
		unsafe {
			self.opcode_without_offset(Opcode::Return);
		}

		#[cfg(debug_assertions)]
		for &opcode in self.code.iter() {
			debug_assert_ne!(opcode, 0, "deferred jump which was never un-deferred encountered.")
		}

		Program {
			code: self.code.into_boxed_slice(),
			constants: self.constants.into_boxed_slice(),
			variables: self.variables,

			#[cfg(feature = "stacktrace")]
			source_lines: self.source_lines,

			#[cfg(feature = "stacktrace")]
			block_locations: self.block_locations,

			_ignored: (&(), &()),
		}
	}

	/// Gets the current index for the program, for use later on with jumps.
	pub fn jump_index(&self) -> JumpIndex {
		JumpIndex(self.code.len())
	}

	/// Indicates that a new line of code, located at `loc`, is about to begin. Used for stacktraces.
	#[cfg(feature = "stacktrace")]
	pub fn record_source_location(&mut self, loc: SourceLocation<'path>) {
		self.source_lines.insert(self.code.len(), loc);
	}

	/// Indicates that at the offset `whence`, a block named `name` with the source location `loc`
	/// exists. Used for stacktraces.
	#[cfg(feature = "stacktrace")]
	pub fn record_block(
		&mut self,
		loc: SourceLocation<'path>,
		whence: JumpIndex,
		name: Option<VariableName<'src>>,
	) {
		self.block_locations.insert(whence, (name, loc));
	}

	/// Writes a jump to `index`, which will only be run if `when` is valid.
	///
	/// This is equivalent to calling `defer_jump` and then immediately calling `jump_to` on it.
	///
	/// # Safety
	/// `index` has to be a valid location to jump to within the program. (This means, but isn't
	/// limited to, jumping out of bounds, or jumping right before a destructive operation like `Add`
	/// isn't allowed. TODO: what other operations are illegal?)
	pub unsafe fn jump_to(&mut self, when: JumpWhen, index: JumpIndex) {
		// SAFETY: TODO
		unsafe { self.defer_jump(when).jump_to(self, index) };
	}

	/// Defers a jump when `when` is complete.
	///
	/// Note that while this itself isn't unsafe, calling [`Compiler::build`] without `.jump_to`ing
	/// the deferred jump is.
	pub fn defer_jump(&mut self, when: JumpWhen) -> DeferredJump {
		let deferred = self.code.len();
		self.code.push(0);
		DeferredJump(deferred, when)
	}

	// SAFETY: `opcode` must take an offset and `offset` must be a valid offset for it.
	pub unsafe fn opcode_with_offset(&mut self, opcode: Opcode, offset: usize) {
		debug_assert!(opcode.takes_offset());

		// No need to check if `offset as InstructionAndOffset`'s topbit is nonzero, as that's so massive it'll never happen
		self.code.push(code_from_opcode_and_offset(opcode, offset))
	}

	// SAFETY: `opcode` mustn't take an offset
	pub unsafe fn opcode_without_offset(&mut self, opcode: Opcode) {
		debug_assert!(!opcode.takes_offset());

		self.code.push(code_from_opcode_and_offset(opcode, 0)) // any offset'll do, it's ignored
	}

	pub fn push_constant(&mut self, value: Value<'gc>) {
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

	fn variable_index(
		&mut self,
		name: VariableName<'src>,
		opts: &Options,
	) -> Result<usize, ParseErrorKind> {
		// TODO: check for name size (also in `set`)
		match self.variables.get_index_of(&name) {
			Some(index) => Ok(index),
			None => {
				let i = self.variables.len();

				#[cfg(feature = "compliance")]
				if opts.compliance.variable_count && i > crate::vm::MAX_VARIABLE_COUNT {
					return Err(ParseErrorKind::TooManyVariables);
				}

				self.variables.insert(name);
				Ok(i)
			}
		}
	}

	pub fn get_variable(
		&mut self,
		name: VariableName<'src>,
		opts: &Options,
	) -> Result<(), ParseErrorKind> {
		let index = self.variable_index(name, opts)?;

		unsafe {
			self.opcode_with_offset(Opcode::GetVar, index);
		}

		Ok(())
	}

	// SAFETY: when called, a value has to be on the stack
	pub unsafe fn set_variable(
		&mut self,
		name: VariableName<'src>,
		opts: &Options,
	) -> Result<(), ParseErrorKind> {
		let index = self.variable_index(name, opts)?;

		unsafe {
			self.opcode_with_offset(Opcode::SetVar, index);
		}

		Ok(())
	}

	// SAFETY: when called, a value has to be on the stack
	#[deprecated(note = "not actually used yet, could be an optimization")]
	pub unsafe fn set_variable_pop(
		&mut self,
		name: VariableName<'src>,
		opts: &Options,
	) -> Result<(), ParseErrorKind> {
		let index = self.variable_index(name, opts)?;

		unsafe {
			self.opcode_with_offset(Opcode::SetVarPop, index);
		}

		Ok(())
	}
}

impl DeferredJump {
	/// Reify `self` by jumping to the current position in `compiler` .
	///
	/// # Safety
	/// Same as [`DeferredJump::jump_to`].
	pub unsafe fn jump_to_current(self, compiler: &mut Compiler<'_, '_, '_>) {
		// SAFETY: TODO
		unsafe { self.jump_to(compiler, compiler.jump_index()) }
	}

	/// Reify `self` by jumping to the position `index` in `compiler`.

	pub unsafe fn jump_to(self, compiler: &mut Compiler<'_, '_, '_>, index: JumpIndex) {
		assert_eq!(0, compiler.code[self.0]);

		let opcode = match self.1 {
			JumpWhen::True => Opcode::JumpIfTrue,
			JumpWhen::False => Opcode::JumpIfFalse,
			JumpWhen::Always => Opcode::Jump,
		};

		compiler.code[self.0] = code_from_opcode_and_offset(opcode, index.0);
	}
}
