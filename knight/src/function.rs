use crate::{Value, Result, Environment, value::Runnable};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy)]
pub struct Function {
	pub name: char,
	pub arity: usize,
	pub func: for<'env> fn(&[Value<'env>], env: &'env mut Environment) -> Result<Value<'env>>
}

impl Debug for Function {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		struct PtrDisp(usize);
		impl Debug for PtrDisp {
			fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
				write!(f, "{:p}", self.0 as *const ())
			}
		}

		if f.alternate() {
			f.debug_struct("Function")
				.field("name", &self.name)
				.field("arity", &self.arity)
				.field("func", &PtrDisp(self.func as usize))
				.finish()
		} else {
			f.debug_tuple("Function")
				.field(&self.name)
				.finish()
		}
	}
}

impl Function {
	pub const fn arity(&self) -> usize {
		self.arity
	}

	pub fn run<'env>(&self, args: &[Value<'env>], env: &'env mut Environment) -> Result<Value<'env>> {
		debug_assert_eq!(self.arity(), args.len());

		(self.func)(args, env)
	}
}

macro_rules! declare_function {
	($static_name:ident, $name:literal, $arity:literal, $body:expr) => {
		pub static $static_name: Function = Function {
			name: $name,
			arity: $arity,
			func: $body
		};
	};
}


declare_function!(NOOP, ':', 1, |args, env| args[0].run(env));

	// RegisterFunction('P', 0, Prompt)
	// RegisterFunction('R', 0, Random)

	// RegisterFunction('E', 1, Eval)
	// RegisterFunction('B', 1, Block)
	// RegisterFunction('C', 1, Call)
	// RegisterFunction('`', 1, System)
	// RegisterFunction('Q', 1, Quit)
	// RegisterFunction('!', 1, Not)
	// RegisterFunction('L', 1, Length)
	// RegisterFunction('D', 1, Dump)
	// RegisterFunction('O', 1, Output)

	// RegisterFunction('+', 2, Add)
	// RegisterFunction('-', 2, Subtract)
	// RegisterFunction('*', 2, Multiply)
	// RegisterFunction('/', 2, Divide)
	// RegisterFunction('%', 2, Modulo)
	// RegisterFunction('^', 2, Exponentiate)
	// RegisterFunction('<', 2, LessThan)
	// RegisterFunction('>', 2, GreaterThan)
	// RegisterFunction('?', 2, EqualTo)
	// RegisterFunction('&', 2, And)
	// RegisterFunction('|', 2, Or)
	// RegisterFunction(';', 2, Then)
	// RegisterFunction('=', 2, Assign)
	// RegisterFunction('W', 2, While)

	// RegisterFunction('I', 3, If)
	// RegisterFunction('G', 3, Get)

	// RegisterFunction('S', 4, Substitute)