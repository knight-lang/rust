use super::{Opcode, ParseErrorKind, SourceLocation};
use crate::options::Options;
use crate::value::KString;
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

#[derive(Default)]
pub struct Builder {
	code: Vec<u64>, // todo: make nonzero u64
	constants: Vec<Value>,
	variables: HashMap<Box<StringSlice>, usize>,

	#[cfg(feature = "knight-debugging")]
	source_lines: HashMap<usize, SourceLocation>,

	#[cfg(feature = "knight-debugging")]
	functions: HashMap<JumpIndex, (Option<KString>, SourceLocation)>,
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

impl DeferredJump {
	pub unsafe fn jump_to_current(self, builder: &mut Builder) {
		// SAFETY: TODO
		unsafe { self.jump_to(builder, builder.jump_index()) }
	}

	pub unsafe fn jump_to(self, builder: &mut Builder, index: JumpIndex) {
		assert_eq!(0, builder.code[self.0]);

		let opcode = match self.1 {
			JumpWhen::True => Opcode::JumpIfTrue,
			JumpWhen::False => Opcode::JumpIfFalse,
			JumpWhen::Always => Opcode::Jump,
		};

		builder.code[self.0] = code_from_opcode_and_offset(opcode, index.0);
	}
}

fn code_from_opcode_and_offset(opcode: Opcode, offset: usize) -> u64 {
	opcode as u64 | (offset as u64) << 0o10
}

impl Builder {
	// needs to be called while ensuring soemthing's ont eh stack.
	pub unsafe fn build(mut self) -> Program {
		unsafe {
			self.opcode_without_offset(Opcode::Return);
		}

		Program {
			code: self.code.into(),
			constants: self.constants.into(),
			num_variables: self.variables.len(),

			#[cfg(feature = "knight-debugging")]
			source_lines: self.source_lines,

			#[cfg(feature = "knight-debugging")]
			functions: self.functions,

			#[cfg(debug_assertions)]
			variable_names: {
				// todo: ordered hash map lol;
				let mut vars = vec![];
				for i in 0..self.variables.len() {
					vars.push(crate::value::KString::default().into_boxed());
				}

				for (name, idx) in self.variables {
					vars[idx] = name;
				}
				vars
			},
		}
	}

	pub fn jump_index(&self) -> JumpIndex {
		JumpIndex(self.code.len())
	}

	#[cfg(feature = "knight-debugging")]
	pub fn record_source_location(&mut self, loc: SourceLocation) {
		self.source_lines.insert(self.code.len(), loc);
	}

	#[cfg(feature = "knight-debugging")]
	pub fn record_function(
		&mut self,
		loc: SourceLocation,
		whence: JumpIndex,
		name: Option<KString>,
	) {
		self.functions.insert(whence, (name, loc));
	}

	// safety, index has to be from this program
	pub unsafe fn jump_to(&mut self, when: JumpWhen, index: JumpIndex) {
		// SAFETY: TODO
		unsafe { self.defer_jump(when).jump_to(self, index) };
	}

	pub fn defer_jump(&mut self, when: JumpWhen) -> DeferredJump {
		let deferred = self.code.len();
		self.code.push(0);
		DeferredJump(deferred, when)
	}

	// SAFETY: `opcode` must take an offset and `offset` must be a valid offset for it.
	unsafe fn opcode_with_offset(&mut self, opcode: Opcode, offset: usize) {
		// No need to check if `offset as u64`'s topbit is nonzero, as that's so massive it'll never happen
		self.code.push(code_from_opcode_and_offset(opcode, offset))
	}

	// SAFETY: `opcode` mustn't take an offset
	pub unsafe fn opcode_without_offset(&mut self, opcode: Opcode) {
		self.code.push(code_from_opcode_and_offset(opcode, 0)) // any offset'll do, it's ignored
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

	fn variable_index(
		&mut self,
		name: &StringSlice,
		opts: &Options,
	) -> Result<usize, ParseErrorKind> {
		#[cfg(feature = "compliance")]
		if opts.compliance.variable_name_length && name.len() > crate::parser::MAX_VARIABLE_LEN {
			return Err(ParseErrorKind::VariableNameTooLong(name.to_owned()));
		}

		// TODO: check for name size (also in `set`)
		match self.variables.get(name) {
			Some(&index) => Ok(index),
			None => {
				let i = self.variables.len();

				#[cfg(feature = "compliance")]
				if opts.compliance.variable_count && i > crate::vm::MAX_VARIABLE_COUNT {
					return Err(ParseErrorKind::TooManyVariables);
				}

				// TODO: check `name` variable len
				self.variables.insert(name.into_boxed(), i);
				Ok(i)
			}
		}
	}

	pub fn get_variable(
		&mut self,
		name: &StringSlice,
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
		name: &StringSlice,
		opts: &Options,
	) -> Result<(), ParseErrorKind> {
		let index = self.variable_index(name, opts)?;

		unsafe {
			self.opcode_with_offset(Opcode::SetVar, index);
		}

		Ok(())
	}

	// SAFETY: when called, a value has to be on the stack
	pub unsafe fn set_variable_pop(
		&mut self,
		name: &StringSlice,
		opts: &Options,
	) -> Result<(), ParseErrorKind> {
		let index = self.variable_index(name, opts)?;

		unsafe {
			self.opcode_with_offset(Opcode::SetVarPop, index);
		}

		Ok(())
	}
}
