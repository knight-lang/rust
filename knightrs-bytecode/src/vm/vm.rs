use std::cmp::Ordering;
use std::mem::MaybeUninit;

use super::{Opcode, Program, RuntimeError};
use crate::strings::StringSlice;
use crate::value::{Block, Integer, List, ToBoolean, ToInteger, ToKString, Value};
use crate::{Environment, Error};

pub struct Vm<'prog, 'env> {
	program: &'prog Program,
	env: &'env mut Environment,
	current_index: usize,
	stack: Vec<Value>,
	vars: Box<[Value]>,
	#[cfg(feature = "stacktrace")]
	call_stack: Vec<Block>,
}

impl<'prog, 'env> Vm<'prog, 'env> {
	pub fn new(program: &'prog Program, env: &'env mut Environment) -> Self {
		Self {
			program,
			env,
			current_index: 0,
			stack: Vec::new(),
			vars: vec![Value::Null; program.num_variables()].into(),
			#[cfg(feature = "stacktrace")]
			call_stack: Vec::new(),
		}
	}

	pub fn child_stackframe(&mut self, block: Block) -> crate::Result<Value> {
		let index = self.current_index;
		let stack_len = self.stack.len();

		self.current_index = block.inner().0;
		self.call_stack.push(block);
		let result = self.run();

		#[cfg(feature = "stacktrace")]
		let result = match result {
			Ok(ok) => Ok(ok),
			Err(todo @ crate::Error::Stacktrace(_)) => Err(todo),
			Err(err) => Err(crate::Error::Stacktrace(self.error(err).to_string())),
		};

		debug_assert_eq!(stack_len, self.stack.len());
		self.current_index = index;
		let popped = self.call_stack.pop();
		debug_assert_eq!(popped, Some(block));

		result
	}

	pub fn error(&mut self, err: crate::Error) -> RuntimeError {
		RuntimeError {
			err,
			#[cfg(feature = "stacktrace")]
			stacktrace: self.stacktrace(),
		}
	}

	#[cfg(feature = "stacktrace")]
	pub fn stacktrace(&self) -> super::Stacktrace {
		super::Stacktrace::new(self.call_stack.iter().map(|block| {
			let (name, loc) = self.program.function_name(*block);
			(name.cloned(), loc.clone())
		}))
	}

	pub fn run(&mut self) -> crate::Result<Value> {
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
					self.stack.push(self.program.constant_at(offset).clone());
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
					self.stack.push(self.vars[offset].clone());
				}

				SetVar => {
					self.vars[offset] = self.stack.last().unwrap().clone();
				}

				SetVarPop => self.vars[offset] = arg![0].clone(),

				// Arity 0
				Prompt => self.stack.push(self.env.prompt()?.map(Value::from).unwrap_or_default()),
				Random => self.stack.push(self.env.random()?.into()),
				Dup => self.stack.push(self.stack.last().unwrap().clone()),
				Return => return Ok(self.stack.pop().unwrap()),

				// Arity 1
				Call => {
					let result = arg![0].kn_call(self)?;
					self.stack.push(result)
				}
				Quit => {
					let status = arg![0].to_integer(self.env)?;
					let status = i32::try_from(status.inner()).expect("todo: out of bounds for i32");

					return Err(Error::Exit(status));
				}
				Dump => {
					arg![0].kn_dump();
					self.stack.push(arg![0].clone());
				}
				Output => {
					println!("{}", arg![0].to_kstring(self.env)?.as_str());
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
