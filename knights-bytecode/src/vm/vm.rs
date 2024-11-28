use super::Opcode;
use crate::value::{Integer, ToKString, Value};
use crate::{Environment, Result};

// Arity 1: :, BLOCK, CALL, QUIT, DUMP, OUTPUT, LENGTH, !, ~, ASCII, ,, [, ]
// Arity 2: +, -, *, /, %, ^, <, >, ?, &, |, ;, =, WHILE
// Arity 3: IF, GET
// Arity 4: SET
// }

pub fn foo() -> Program {
	Program {
		code: vec![
			Opcode::PushConstant as u8,
			0,
			Opcode::Output as u8,
			Opcode::Pop as u8,
			Opcode::PushConstant as u8,
			1,
			Opcode::Return as u8,
		]
		.into(),
		constants: vec![Value::Boolean(true), Value::Integer(Integer::ZERO)].into(),
		num_variables: 0,
	}
}

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
}

impl<'p, 'e> Vm<'p, 'e> {
	pub fn new(program: &'p Program, env: &'e mut Environment) -> Self {
		Self { program, env, current_index: 0, stack: Vec::new() }
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
			let mut args: [Value; Opcode::MAX_ARITY] =
				[Value::Null, Value::Null, Value::Null, Value::Null];

			let offset = if opcode.takes_offset() {
				self.next_usize()
			} else {
				0 // TODO: maybeuninit
			};

			// TODO: do we need to reverse?
			for idx in 0..opcode.arity() {
				args[idx] = self.stack.pop().unwrap();
			}

			match opcode {
				// Builtins
				Opcode::PushConstant => {
					self.stack.push(self.program.constants[offset].clone());
				}

				Opcode::Jump => {
					todo!()
				}

				Opcode::JumpIfTrue => {
					todo!()
				}

				Opcode::JumpIfFalse => {
					todo!()
				}

				Opcode::GetVar => {
					todo!()
				}

				Opcode::SetVar => {
					todo!()
				}

				Opcode::Pop => { /* do nothing, the arity already popped */ }
				Opcode::Output => {
					println!("{}", args[0].to_kstring(self.env)?.as_str());
					self.stack.push(Value::Null);
				}
				Opcode::Quit => {
					todo!()
				}
				// (Opcode::Output, 0, 0),
				// (Opcode::Pop, 0, 0),
				// (Opcode::PushConstant, 1, 0),
				// (Opcode::Quit, 0, 0),
				Opcode::Set => {
					todo!()
					// let other = self.stack.pop();
				}
				_ => todo!(),
			}
		}

		Ok(self.stack.pop().unwrap())
	}
}
