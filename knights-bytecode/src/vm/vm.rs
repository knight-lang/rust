use super::{Opcode, Program};
use crate::value::{Integer, ToBoolean, ToInteger, ToKString, Value};
use crate::{Environment, Result};

pub struct Vm<'prog, 'env> {
	program: &'prog Program,
	env: &'env mut Environment,
	current_index: usize,
	stack: Vec<Value>,
	vars: Box<[Value]>,
}

impl<'prog, 'env> Vm<'prog, 'env> {
	pub fn new(program: &'prog Program, env: &'env mut Environment) -> Self {
		Self {
			program,
			env,
			current_index: 0,
			stack: Vec::new(),
			vars: vec![Value::Null; program.num_variables()].into(),
		}
	}

	pub fn run(&mut self) -> Result<Value> {
		use Opcode::*;

		loop {
			let (opcode, offset) = self.program.opcode_at(self.current_index);
			self.current_index += 1;

			// println!("{:?}: {:?} / {:?}: {:?}", self.current_index, offset, opcode, self.vars);
			let mut args: [Value; Opcode::MAX_ARITY] =
				[Value::Null, Value::Null, Value::Null, Value::Null];

			// TODO: do we need to reverse?
			for idx in (0..opcode.arity()).rev() {
				args[idx] = self.stack.pop().unwrap();
			}

			match opcode {
				_Invalid => unreachable!(),

				// Builtins
				PushConstant => {
					self.stack.push(self.program.constant_at(offset).clone());
				}

				Jump => {
					self.current_index = offset;
				}

				JumpIfTrue => {
					if args[0].to_boolean(self.env)? {
						self.current_index = offset;
					}
				}

				JumpIfFalse => {
					if !args[0].to_boolean(self.env)? {
						self.current_index = offset;
					}
				}

				GetVar => {
					self.stack.push(self.vars[offset].clone());
				}

				SetVar => {
					self.vars[offset] = self.stack.last().unwrap().clone();
				}

				SetVarPop => self.vars[offset] = args[0].clone(),

				// Arity 0
				Prompt => todo!(),
				Random => todo!(),
				Dup => self.stack.push(self.stack.last().unwrap().clone()),
				Return => return Ok(self.stack.pop().unwrap()),

				// Arity 1
				Call => todo!(),
				Quit => todo!(),
				Dump => todo!(),
				// Output => todo!(),
				Output => {
					println!("{}", args[0].to_kstring(self.env)?.as_str());
					self.stack.push(Value::Null);
				}
				Length => todo!(),
				Not => todo!(),
				Negate => todo!(),
				Ascii => todo!(),
				Box => todo!(),
				Head => todo!(),
				Tail => todo!(),
				Pop => { /* do nothing, the arity already popped */ }

				// Arity 2
				Add => self.stack.push(args[0].op_plus(&args[1], self.env)?),
				Sub => self.stack.push(args[0].op_minus(&args[1], self.env)?),
				Mul => self.stack.push(args[0].op_asterisk(&args[1], self.env)?),
				Div => self.stack.push(args[0].op_slash(&args[1], self.env)?),
				Mod => self.stack.push(args[0].op_percent(&args[1], self.env)?),
				Pow => self.stack.push(args[0].op_caret(&args[1], self.env)?),
				Lth => todo!(),
				Gth => todo!(),
				Eql => todo!(),

				// Arity 3
				Get => todo!(),

				// Arity 4
				Set => todo!(),
			}
		}
	}
}
