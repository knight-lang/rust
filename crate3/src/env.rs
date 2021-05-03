mod variable;
pub use variable::*;

use std::collections::HashSet;
use std::io::{self, BufRead, Write};

use crate::{Result, Text, Value};

pub struct Environment<'i, 'o, 's> {
	// We use a `HashSet` because we want the variable to own its name, which a `HashMap` wouldn't allow for. (or would
	// have redundant allocations.)
	vars: HashSet<Variable<'static>>, // technically, `'self`, but that doesn't exist because dumb rust.
	stdin: &'i mut dyn BufRead,
	stdout: &'o mut dyn Write,
	system: &'s mut dyn FnMut(&str) -> Result<Text>
}

impl<'i, 'o, 's> Environment<'i, 'o, 's> {
	pub fn stdin(&mut self) -> &mut dyn BufRead {
		self.stdin
	}

	pub fn stdout(&mut self) -> &mut dyn Write {
		self.stdout
	}

	pub fn system<S: AsRef<str>>(&mut self, input: &S) -> Result<Text> {
		(self.system)(input.as_ref())
	}

	pub fn eval<'env, S: AsRef<str>>(&'env mut self, input: S) -> Result<Value<'env>> {
		todo!()
	}

	// pub fn variable(name: impl Borrow<str> + )

}
