use super::Opcode;
use crate::value::{Integer, ToBoolean, ToInteger, ToKString, Value};
use crate::{Environment, Result};

// Arity 1: :, BLOCK, CALL, QUIT, DUMP, OUTPUT, LENGTH, !, ~, ASCII, ,, [, ]
// Arity 2: +, -, *, /, %, ^, <, >, ?, &, |, ;, =, WHILE
// Arity 3: IF, GET
// Arity 4: SET
// }
#[rustfmt::skip]
pub fn foo() -> Program {
	fn op(opcode: Opcode, offset: u32) -> u32 {
		((opcode as u8 as u32)) | (offset << 0o10)
	}
	use Opcode::*;
	Program {
		/*
		; = n 10 var=1
		; = i 0 var=0
		; WHILE n
			; = i + i n
			: = n - n 1
		OUTPUT i
					*/
		code: vec![
			op(PushConstant, 2),
			op(SetVarPop, 0),
			op(PushConstant, 0),
			op(SetVarPop, 1),
			// WHILE
			op(GetVar, 0),
			op(JumpIfFalse, 15),
			op(GetVar, 0),
			op(GetVar, 1),
			op(Add, 0xff),
			op(SetVarPop, 1),
			op(PushConstant, 1),
			op(GetVar, 0),
			op(Sub, 0xff),
			op(SetVarPop, 0),
			op(Jump, 4),
			op(GetVar, 1),
			op(Output, 0xff),
			op(Return, 0xff) ,
		]
		.into(),
		constants: vec![
			Value::Integer(Integer::new_unvalidated(0)),
			Value::Integer(Integer::new_unvalidated(1)),
			Value::Integer(Integer::new_unvalidated(10)),
		]
		.into(),
		num_variables: 2,
	}
}

pub struct Program {
	code: Box<[u32]>,
	constants: Box<[Value]>,
	num_variables: usize,
}

pub struct Vm<'p, 'e> {
	program: &'p Program,
	env: &'e mut Environment,
	current_index: usize,
	stack: Vec<Value>,
	vars: Box<[Value]>,
}

impl<'p, 'e> Vm<'p, 'e> {
	pub fn new(program: &'p Program, env: &'e mut Environment) -> Self {
		Self {
			program,
			env,
			current_index: 0,
			stack: Vec::new(),
			vars: vec![Value::Null; program.num_variables].into(),
		}
	}

	fn next_opcode(&mut self) -> (Opcode, usize) {
		let number = self.program.code[self.current_index];
		self.current_index += 1;

		// SAFETY: we know as this type was constructed that all programs result
		// in valid opcodes
		let opcode = unsafe { Opcode::from_byte_unchecked((number as u8)) };
		let offset = (number >> 0o10) as usize;

		(opcode, offset)
	}

	pub fn run(&mut self) -> Result<Value> {
		use Opcode::*;

		loop {
			let (opcode, offset) = self.next_opcode();
			// println!("{:?}: {:?} / {:?}", self.current_index, offset, opcode);
			let mut args: [Value; Opcode::MAX_ARITY] =
				[Value::Null, Value::Null, Value::Null, Value::Null];

			// TODO: do we need to reverse?
			for idx in 0..opcode.arity() {
				args[idx] = self.stack.pop().unwrap();
			}

			match opcode {
				// Builtins
				PushConstant => {
					self.stack.push(self.program.constants[offset].clone());
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
				Dup => todo!(),
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
