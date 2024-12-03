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
		const NULL: Value = Value::Null;

		use Opcode::*;

		loop {
			// SAFETY: all programs are well-formed, so we know the current index is in bounds.
			let (opcode, offset) = unsafe { self.program.opcode_at(self.current_index) };
			self.current_index += 1;

			// println!("{:?}: {:?} / {:?}: {:?}", self.current_index, offset, opcode, self.vars);
			let mut args = [NULL; Opcode::MAX_ARITY];

			// TODO: do we need to reverse?
			// debug_assert!(opcode.arity() <= self.stack.len());
			// std::ptr::copy_nonoverlapping(
			// 	self
			// 		.stack
			// 		.as_mut_ptr()
			// 		.offset(self.stack.len() as isize - opcode.arity() as isize)
			// 		.cast::<MaybeUninit<Value>>(),
			// 	args.as_mut_ptr(),
			// 	opcode.arity(),
			// );

			for idx in (0..opcode.arity()).rev() {
				args[idx] = self.stack.pop().unwrap();
			}

			macro_rules! arg {
				($idx:expr) => {{
					let idx = $idx;
					debug_assert!(idx < opcode.arity());
					&args[idx]
				}};
			}

			match opcode {
				// Builtins
				PushConstant => {
					self.stack.push(unsafe { self.program.constant_at(offset) }.clone());
				}

				Jump => {
					self.current_index = offset;
				}

				JumpIfTrue => {
					if arg![0].to_boolean(self.env)? {
						self.current_index = offset;
					}
				}

				JumpIfFalse => {
					if !arg![0].to_boolean(self.env)? {
						self.current_index = offset;
					}
				}

				GetVar => {
					self
						.stack
						.push(self.vars[offset].as_ref().expect("todo: UndefinedVariable").clone());
				}

				SetVar => {
					let value = self.stack.last().unwrap().clone();

					#[cfg(feature = "stacktrace")]
					if let Value::Block(ref block) = value {
						let varname = self.program.variable_name(offset);
						self.known_blocks.insert(block.inner().0, varname);
					}

					self.vars[offset] = Some(value);
				}

				SetVarPop => todo!(), //self.vars[offset] = arg![0].clone(),

				// Arity 0
				Prompt => self.stack.push(self.env.prompt()?.map(Value::from).unwrap_or_default()),
				Random => self.stack.push(self.env.random()?.into()),
				Dup => self.stack.push(self.stack.last().unwrap().clone()),
				Return => return Ok(self.stack.pop().unwrap()),

				// Arity 1
				Call => {
					let result = arg![0].kn_call(self)?;
					self.stack.push(result);
				}
				Quit => {
					let status = arg![0].to_integer(self.env)?;
					let status = i32::try_from(status.inner()).expect("todo: out of bounds for i32");
					self.env.quit(status)?;
				}
				Dump => {
					arg![0].kn_dump(self.env)?;
					self.stack.push(arg![0].clone());
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

					self.stack.push(Value::Null);
				}
				Length => self.stack.push(arg![0].kn_length(self.env)?.into()),
				Not => self.stack.push((!arg![0].to_boolean(self.env)?).into()),
				Negate => self.stack.push(arg![0].kn_negate(self.env)?.into()),
				Ascii => self.stack.push(arg![0].kn_ascii(self.env)?),
				Box => self.stack.push(List::boxed(arg![0].clone()).into()),
				Head => self.stack.push(arg![0].kn_head(self.env)?),
				Tail => self.stack.push(arg![0].kn_tail(self.env)?),
				Pop => { /* do nothing, the arity already popped */ }

				// TODO: the `vm` evals in its entirely own vm, which isnt what we wnat
				#[cfg(feature = "extensions")]
				Eval => {
					let program = arg![0].to_kstring(self.env)?;
					let mut parser = crate::parser::Parser::new(&mut self.env, None, program.as_str())?;
					let program = parser.parse_program()?;
					self.stack.push(Vm::new(&program, self.env).run_entire_program()?);
				}

				// Arity 2
				Add => self.stack.push(arg![0].kn_plus(arg![1], self.env)?),
				Sub => self.stack.push(arg![0].kn_minus(arg![1], self.env)?),
				Mul => self.stack.push(arg![0].kn_asterisk(arg![1], self.env)?),
				Div => self.stack.push(arg![0].kn_slash(arg![1], self.env)?),
				Mod => self.stack.push(arg![0].kn_percent(arg![1], self.env)?),
				Pow => self.stack.push(arg![0].kn_caret(arg![1], self.env)?),
				Lth => self
					.stack
					.push((arg![0].kn_compare(arg![1], "<", self.env)? == Ordering::Less).into()),
				Gth => self
					.stack
					.push((arg![0].kn_compare(arg![1], ">", self.env)? == Ordering::Greater).into()),
				Eql => self.stack.push((arg![0].kn_equals(arg![1], self.env)?).into()),

				// Arity 3
				Get => self.stack.push(arg![0].kn_get(arg![1], arg![2], self.env)?),

				// Arity 4
				Set => self.stack.push(arg![0].kn_set(arg![1], arg![2], arg![3], self.env)?),
			}
		}
	}
}
