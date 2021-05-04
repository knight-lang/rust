mod variable;
mod builder;

pub use builder::*;
pub use variable::*;

use std::collections::HashSet;
use std::io::{self, Read, BufReader, BufRead, Write};
use std::cell::RefCell;

use crate::{Result, Text, Value};

pub struct Environment<'i, 'o, 's> {
	// We use a `HashSet` because we want the variable to own its name, which a `HashMap` wouldn't
	// allow for. (or would ave redundant allocations.)
	vars: HashSet<Variable>,
	stdin: BufReader<&'i mut dyn Read>,
	stdout: &'o mut dyn Write,
	system: &'s mut dyn FnMut(&str) -> Result<Text>
}

impl Default for Environment<'static, 'static, 'static> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'i, 'o, 's> Environment<'i, 'o, 's> {
	pub fn new() -> Self {
		Self::builder().build()
	}

	pub fn builder() -> Builder<'i, 'o, 's> {
		Builder::default()
	}

	pub fn system<S: AsRef<str>>(&mut self, input: &S) -> Result<Text> {
		(self.system)(input.as_ref())
	}

	pub fn eval<S: AsRef<str>>(&mut self, input: S) -> Result<Value> {
		todo!()
	}

	pub fn variable(&mut self, name: &str) -> Variable {
		if let Some(var) = self.vars.get(name) {
			return var.clone();
		}

		let var = Variable::create(name.to_string());
		self.vars.insert(var.clone());
		var
	}
}

impl Read for Environment<'_, '_, '_> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.stdin.read(buf)
	}
}

impl BufRead for Environment<'_, '_, '_> {
	fn fill_buf(&mut self) -> io::Result<&[u8]> {
		self.stdin.fill_buf()
	}

	fn consume(&mut self, amnt: usize) {
		self.stdin.consume(amnt)
	}

	fn read_line(&mut self, out: &mut String) -> io::Result<usize> {
		self.stdin.read_line(out)
	}
}

impl Write for Environment<'_, '_, '_> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.stdout.write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.stdout.flush()
	}
}
