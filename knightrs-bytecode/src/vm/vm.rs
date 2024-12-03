use std::cmp::Ordering;
use std::collections::HashMap;
use std::mem::MaybeUninit;

use super::{Opcode, RuntimeError};
use crate::parser::{SourceLocation, VariableName};
use crate::program::{JumpIndex, Program};
use crate::strings::StringSlice;
use crate::value::{Block, Integer, List, ToBoolean, ToInteger, ToKString, Value};
use crate::{Environment, Error};

pub struct Vm<'prog, 'src, 'path, 'env> {
	program: &'prog Program<'src, 'path>,
	env: &'env mut Environment,
	current_index: usize,
	stack: Vec<Value>,
	vars: Box<[Option<Value>]>,

	#[cfg(feature = "stacktrace")]
	callstack: Vec<usize>,

	#[cfg(feature = "stacktrace")]
	known_blocks: std::collections::HashMap<usize, VariableName<'src>>,
}

impl<'prog, 'src, 'path, 'env> Vm<'prog, 'src, 'path, 'env> {
	pub fn new(program: &'prog Program<'src, 'path>, env: &'env mut Environment) -> Self {
		Self {
			program,
			env,
			current_index: 0,
			stack: Vec::new(),
			vars: vec![None; program.num_variables()].into(),

			#[cfg(feature = "stacktrace")]
			callstack: Vec::new(),

			#[cfg(feature = "stacktrace")]
			known_blocks: Default::default(),
		}
	}

	pub fn run_entire_program(&mut self) -> crate::Result<Value> {
		self.run(Block::new(JumpIndex(0)))
	}

	pub fn run(&mut self, block: Block) -> crate::Result<Value> {
		// Save previous index
		let index = self.current_index;
		#[cfg(feature = "stacktrace")]
		self.callstack.push(self.current_index);

		// Used for debugging later
		let stack_len = self.stack.len();

		// Actually call the functoin
		self.current_index = block.inner().0;
		let result = self.run_inner();

		// Add the stacktrace to the lsit
		#[cfg(feature = "stacktrace")]
		let result = match result {
			Ok(ok) => Ok(ok),
			Err(todo @ crate::Error::Stacktrace(_)) => Err(todo),
			Err(err) => Err(crate::Error::Stacktrace(self.error(err).to_string())),
		};

		#[cfg(feature = "stacktrace")]
		{
			let result = self.callstack.pop();
			debug_assert_eq!(result, Some(index));
		}

		debug_assert_eq!(stack_len, self.stack.len());
		self.current_index = index;

		result
	}

	pub fn error(&mut self, err: crate::Error) -> RuntimeError {
		RuntimeError {
			err,
			#[cfg(feature = "stacktrace")]
			stacktrace: self.stacktrace(),
			_ignored: (&(), &()),
		}
	}

	#[cfg(feature = "stacktrace")]
	pub fn stacktrace(&self) -> super::Stacktrace {
		use super::Callsite;

		super::Stacktrace::new(self.callstack.iter().map(|&idx| {
			let loc = self.program.source_location_at(idx);
			Callsite::new(self.block_name_at(idx), loc)
		}))
	}

	#[cfg(feature = "stacktrace")]
	fn block_name_at(&self, mut idx: usize) -> Option<VariableName> {
		while idx != 0 {
			if let Some(&name) = self.known_blocks.get(&idx) {
				return Some(name.clone());
			}

			idx -= 1;
		}

		None
	}

	pub fn run_inner(&mut self) -> crate::Result<Value> {
		use std::mem::MaybeUninit;
		const NULL: MaybeUninit<Value> = MaybeUninit::uninit();

		use Opcode::*;

		let mut args = [NULL; Opcode::MAX_ARITY];

		loop {
			// SAFETY: all programs are well-formed, so we know the current index is in bounds.
			let (opcode, offset) = unsafe { self.program.opcode_at(self.current_index) };
			self.current_index += 1;
			// println!("{:?}: {:?} / {:?}: {:?}", self.current_index, offset, opcode, self.vars);

			// Read arguments in
			unsafe {
				debug_assert!(opcode.arity() <= self.stack.len());

				// // Copy arguments from the stack into the arguments buffer
				// args.as_mut_ptr().copy_from_nonoverlapping(
				// 	self
				// 		.stack
				// 		.as_mut_ptr()
				// 		.offset(self.stack.len() as isize - opcode.arity() as isize)
				// 		.cast(),
				// 	opcode.arity(),
				// );

				// Pop the arguments off the stack.
				self.stack.set_len(self.stack.len() - opcode.arity());
			}

			let args = self.stack.spare_capacity_mut();

			macro_rules! arg {
				(& $idx:expr) => {{
					let idx = $idx;
					debug_assert!(idx < opcode.arity());
					unsafe { args[idx].assume_init_ref() }
				}};
				($idx:expr) => {{
					let idx = $idx;
					debug_assert!(idx < opcode.arity());
					unsafe { args[idx].assume_init_read() }
				}};
			}

			// NOTE: ALL OPCODES MUST ALWAYS EXTRACT THEIR ARGUMENTS EXACTLY ONCE FROM `args`,
			// else memory issues will crop up (such as memory leaks or double reads).
			let value = match opcode {
				// Builtins
				PushConstant => unsafe { self.program.constant_at(offset) }.clone(),

				Jump => {
					self.current_index = offset;
					continue;
				}

				JumpIfTrue => {
					if arg![0].to_boolean(self.env)? {
						self.current_index = offset;
					}
					continue;
				}

				JumpIfFalse => {
					if !arg![0].to_boolean(self.env)? {
						self.current_index = offset;
					}
					continue;
				}

				GetVar => self.vars[offset].as_ref().expect("todo: UndefinedVariable").clone(),

				SetVar => {
					let value = self.stack.last().unwrap().clone();

					#[cfg(feature = "stacktrace")]
					if let Value::Block(ref block) = value {
						let varname = self.program.variable_name(offset);
						self.known_blocks.insert(block.inner().0, varname);
					}

					self.vars[offset] = Some(value);
					continue;
				}

				SetVarPop => todo!(), //self.vars[offset] = arg![0].clone(),

				// Arity 0
				Prompt => self.env.prompt()?.map(Value::from).unwrap_or_default(),
				Random => self.env.random()?.into(),
				Dup => self.stack.last().unwrap().clone(),
				Return => return Ok(self.stack.pop().unwrap()),

				// Arity 1
				Call => {
					let result = arg![0].kn_call(self)?;
					result
				}
				Quit => {
					let status = arg![0].to_integer(self.env)?;
					let status = i32::try_from(status.inner()).expect("todo: out of bounds for i32");
					self.env.quit(status)?;
					unreachable!()
				}
				Dump => {
					arg![0].kn_dump(self.env)?;
					arg![0].clone()
				}
				Output => {
					use std::io::Write;
					let kstring = arg![0].to_kstring(self.env)?;
					let strref = kstring.as_str();

					let mut output = self.env.output();

					if let Some(stripped) = strref.strip_suffix('\\') {
						write!(output, "{stripped}")
					} else {
						writeln!(output, "{strref}")
					}
					.map_err(|err| Error::IoError { func: "OUTPUT", err })?;

					Value::Null
				}
				Length => arg![0].kn_length(self.env)?.into(),
				Not => (!arg![0].to_boolean(self.env)?).into(),
				Negate => arg![0].kn_negate(self.env)?.into(),
				Ascii => arg![0].kn_ascii(self.env)?,
				Box => List::boxed(arg![0].clone()).into(),
				Head => arg![0].kn_head(self.env)?,
				Tail => arg![0].kn_tail(self.env)?,
				Pop => continue, /* do nothing, the arity already popped */

				// TODO: the `vm` evals in its entirely own vm, which isnt what we wnat
				#[cfg(feature = "extensions")]
				Eval => {
					let program = arg![0].to_kstring(self.env)?;
					let mut parser = crate::parser::Parser::new(&mut self.env, None, program.as_str())?;
					let program = parser.parse_program()?;
					Vm::new(&program, self.env).run_entire_program()?
				}

				// Arity 2
				Add => arg![0].kn_plus(&arg![1], self.env)?,
				Sub => arg![0].kn_minus(&arg![1], self.env)?,
				Mul => arg![0].kn_asterisk(&arg![1], self.env)?,
				Div => arg![0].kn_slash(&arg![1], self.env)?,
				Mod => arg![0].kn_percent(&arg![1], self.env)?,
				Pow => arg![0].kn_caret(&arg![1], self.env)?,
				Lth => (arg![0].kn_compare(&arg![1], "<", self.env)? == Ordering::Less).into(),
				Gth => (arg![0].kn_compare(&arg![1], ">", self.env)? == Ordering::Greater).into(),
				Eql => (arg![0].kn_equals(&arg![1], self.env)?).into(),

				// Arity 3
				Get => arg![0].kn_get(&arg![1], &arg![2], self.env)?,

				// Arity 4
				Set => arg![0].kn_set(&arg![1], &arg![2], &arg![3], self.env)?,
			};
			self.stack.push(value);
		}
	}
}
