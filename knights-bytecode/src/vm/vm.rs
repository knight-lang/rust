use super::Opcode;
use crate::{
	env::Env,
	value::{Integer, Value},
	Result,
};

// Arity 1: :, BLOCK, CALL, QUIT, DUMP, OUTPUT, LENGTH, !, ~, ASCII, ,, [, ]
// Arity 2: +, -, *, /, %, ^, <, >, ?, &, |, ;, =, WHILE
// Arity 3: IF, GET
// Arity 4: SET
// }

pub fn foo() -> Program {
	Program {
		code: vec![
			(Opcode::PushConstant, 0, 0),
			(Opcode::Output, 0, 0),
			(Opcode::Pop, 0, 0),
			(Opcode::PushConstant, 1, 0),
			(Opcode::Quit, 0, 0),
		]
		.into(),
		constants: vec![Value::Boolean(true), Value::Integer(Integer::ZERO)].into(),
		num_variables: 0,
	}
}

pub struct Program {
	code: Box<[(Opcode, u8, u16)]>,
	constants: Box<[Value]>,
	num_variables: usize,
}

pub struct Vm<'p, 'e> {
	program: &'p Program,
	env: &'e Env,
	current_index: usize,
	stack: Vec<Value>,
}

impl<'p, 'e> Vm<'p, 'e> {
	pub fn new(program: &'p Program, env: &'e Env) -> Self {
		Self { program, env, current_index: 0, stack: Vec::new() }
	}

	pub fn run(&mut self) -> Result<Value> {
		while self.current_index < self.program.code.len() {
			let (opcode, idx, _) = self.program.code[self.current_index];
			self.current_index += 1;

			let mut args = [Value::Null, Value::Null, Value::Null, Value::Null];

			// TODO: do we need to reverse?
			for idx in 0..opcode.arity() {
				args[idx] = self.stack.pop().unwrap();
			}

			match opcode {
				Opcode::PushConstant => {
					self.stack.push(self.program.constants[idx as usize].clone());
				}
				Opcode::Pop => { /* do nothing, the arity already popped */ }
				Opcode::Output => {
					println!("{}", args[0].to_string(self.env)?.as_str());
					self.stack.push(Value::Null);
				}
				Opcode::Quit => {
					todo!()
				}
				// (Opcode::Output, 0, 0),
				// (Opcode::Pop, 0, 0),
				// (Opcode::PushConstant, 1, 0),
				// (Opcode::Quit, 0, 0),
				_ => todo!(),
			}
		}

		Ok(self.stack.pop().unwrap())
	}
}
