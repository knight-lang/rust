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
			PushConstant as u8, 2,
			SetVarPop as u8, 0,
			PushConstant as u8, 0,
			SetVarPop as u8, 1,
			// WHILE
			GetVar as u8, 0,
			JumpIfFalse as u8, 28,
			GetVar as u8, 0,
			GetVar as u8, 1,
			Add as u8,
			SetVarPop as u8, 1,
			PushConstant as u8, 1,
			GetVar as u8, 0,
			Sub as u8,
			SetVarPop as u8, 0,
			Jump as u8, 8,
			GetVar as u8, 1,
			Output as u8,
			Return as u8, 
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

// pub fn foo() -> Program {
// 	Program {
// 		/*
// 			*/
// 		code: vec![
// 			Opcode::PushConstant as u8,
// 			2,
// 			Opcode::PushConstant as u8,
// 			3,
// 			Opcode::Add as u8,
// 			Opcode::SetVar as u8,
// 			0,
// 			Opcode::Pop as u8,
// 			Opcode::PushConstant as u8,
// 			0,
// 			Opcode::SetVar as u8,
// 			1,
// 			Opcode::Pop as u8,
// 			Opcode::GetVar as u8,
// 			1,
// 			Opcode::GetVar as u8,
// 			0,
// 			Opcode::Add as u8,
// 			Opcode::Output as u8,
// 			// Opcode::PushConstant as u8,
// 			// 1,
// 			Opcode::Return as u8,
// 		]
// 		.into(),
// 		constants: vec![
// 			Value::Boolean(true),
// 			Value::Integer(Integer::ZERO),
// 			Value::Integer(Integer::new_unvalidated(123)),
// 			Value::Integer(Integer::new_unvalidated(456)),
// 		]
// 		.into(),
// 		num_variables: 2,
// 	}
// }

pub struct Program {
	code: Box<[u8]>,
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

	fn next_byte(&mut self) -> u8 {
		let byte = self.program.code[self.current_index];
		self.current_index += 1;
		byte
	}

	fn next_opcode(&mut self) -> Opcode {
		let byte = self.next_byte();

		// SAFETY: we know as this type was constructed that all programs result
		// in valid opcodes
		unsafe { Opcode::from_byte_unchecked(byte) }
	}

	fn next_usize(&mut self) -> usize {
		let byte = self.next_byte();
		if byte != 0xff {
			return byte as usize;
		}

		// TODO: is this right?
		((self.next_byte() as usize) << 0o30)
			| ((self.next_byte() as usize) << 0o20)
			| ((self.next_byte() as usize) << 0o10)
			| ((self.next_byte() as usize) << 0o00)
	}

	pub fn run(&mut self) -> Result<Value> {
		loop {
			let opcode = self.next_opcode();
			// println!("{:?}: {:?}", self.current_index, opcode);
			let mut args: [Value; Opcode::MAX_ARITY] =
				[Value::Null, Value::Null, Value::Null, Value::Null];

			let offset = if opcode.takes_offset() {
				self.next_usize()
			} else {
				11111 // TODO: maybeuninit
			};

			// TODO: do we need to reverse?
			for idx in 0..opcode.arity() {
				args[idx] = self.stack.pop().unwrap();
			}

			match opcode {
				Opcode::Return => break,

				// Builtins
				Opcode::PushConstant => {
					self.stack.push(self.program.constants[offset].clone());
				}

				Opcode::Jump => {
					self.current_index = offset;
				}

				Opcode::JumpIfTrue => {
					todo!()
				}

				Opcode::JumpIfFalse => {
					if !args[0].to_boolean(self.env)? {
						self.current_index = offset;
					}
				}

				Opcode::GetVar => {
					self.stack.push(self.vars[offset].clone());
				}

				Opcode::SetVar => {
					self.vars[offset] = self.stack.last().unwrap().clone();
				}

				Opcode::SetVarPop => self.vars[offset] = args[0].clone(),

				Opcode::Pop => { /* do nothing, the arity already popped */ }
				Opcode::Output => {
					println!("{}", args[0].to_kstring(self.env)?.as_str());
					self.stack.push(Value::Null);
				}
				Opcode::Quit => {
					todo!()
				}
				Opcode::Add => match args[0] {
					Value::Integer(int) => {
						self.stack.push(int.add(args[1].to_integer(self.env)?, self.env.opts())?.into())
					}
					_ => todo!("add {:?}", args[0]),
				},
				Opcode::Sub => match args[0] {
					Value::Integer(int) => self
						.stack
						.push(int.subtract(args[1].to_integer(self.env)?, self.env.opts())?.into()),
					_ => todo!("add {:?}", args[0]),
				},
				// (Opcode::Output, 0, 0),
				// (Opcode::Pop, 0, 0),
				// (Opcode::PushConstant, 1, 0),
				// (Opcode::Quit, 0, 0),
				Opcode::Set => {
					todo!()
					// let other = self.stack.pop();
				}
				_ => todo!("{:?}", opcode),
			}
		}

		Ok(self.stack.pop().unwrap())
	}
}
