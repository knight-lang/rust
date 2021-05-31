use crate::{Value, Result, Environment};
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