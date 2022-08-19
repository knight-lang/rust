use crate::{Error, KnightStr, Result, Text, Value};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::rc::Rc;

type SystemCommand = dyn FnMut(&str) -> Result<Text>;

#[derive(Default)]
pub struct Environment<'a> {
	// We use a `HashSet` because we want the variable to own its name, which a `HashMap`
	// wouldn't allow for. (or would have redundant allocations.)
	variables: HashSet<Variable>,
	stdin: Option<BufReader<&'a mut dyn Read>>,
	stdout: Option<&'a mut dyn Write>,
	system: Option<&'a mut SystemCommand>,
}

impl Environment<'_> {
	/// Fetches the variable corresponding to `name` in the environment, creating one if it's the
	/// first time that name has been requested
	pub fn lookup(&mut self, name: &KnightStr) -> Variable {
		if let Some(var) = self.variables.get(name) {
			return var.clone();
		}

		let variable = Variable(Rc::new((name.to_boxed(), RefCell::new(None))));
		self.variables.insert(variable.clone());
		variable
	}
}

impl Read for Environment<'_> {
	/// Read bytes into `data` from `self`'s `stdin`.
	///
	/// The `stdin` can be customized at creation via [`Builder::stdin`].
	fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
		if let Some(ref mut stdin) = self.stdin {
			stdin.read(data)
		} else {
			std::io::stdin().read(data)
		}
	}
}

impl BufRead for Environment<'_> {
	fn fill_buf(&mut self) -> io::Result<&[u8]> {
		if let Some(ref mut stdin) = self.stdin {
			stdin.fill_buf()
		} else {
			todo!()
			// std::io::stdin().fill_buf()
		}
	}

	fn consume(&mut self, amnt: usize) {
		if let Some(ref mut stdin) = self.stdin {
			stdin.consume(amnt)
		} else {
			todo!()
			// std::io::stdin().consume(amnt)
		}
	}

	fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
		if let Some(ref mut stdin) = self.stdin {
			stdin.read_line(buf)
		} else {
			std::io::stdin().read_line(buf)
		}
	}
}

impl Write for Environment<'_> {
	fn write(&mut self, data: &[u8]) -> io::Result<usize> {
		if let Some(ref mut stdout) = self.stdout {
			stdout.write(data)
		} else {
			std::io::stdout().write(data)
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		if let Some(ref mut stdout) = self.stdout {
			stdout.flush()
		} else {
			std::io::stdout().flush()
		}
	}
}

impl std::borrow::Borrow<KnightStr> for Variable {
	fn borrow(&self) -> &KnightStr {
		self.name()
	}
}

#[derive(Clone)]
pub struct Variable(Rc<(Box<KnightStr>, RefCell<Option<Value>>)>);

impl Eq for Variable {}
impl PartialEq for Variable {
	/// Checks to see if two variables are equal.
	///
	/// This'll just check to see if their names are equivalent. Techincally, this means that
	/// two variables with the same name, but derived from different [`Environment`]s will end up
	/// being the same
	fn eq(&self, rhs: &Self) -> bool {
		(self.0).0 == (rhs.0).0
	}
}

impl Hash for Variable {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(self.0).0.hash(state)
	}
}

impl Variable {
	/// Fetches the name of the variable.
	#[must_use]
	pub fn name(&self) -> &KnightStr {
		&(self.0).0
	}

	/// Assigns a new value to the variable, returning whatever the previous value was.
	pub fn assign(&self, new: Value) -> Option<Value> {
		(self.0).1.replace(Some(new))
	}

	/// Fetches the last value assigned to `self`, returning `None` if we haven't been assigned to yet.
	#[must_use]
	pub fn fetch(&self) -> Option<Value> {
		(self.0).1.borrow().clone()
	}

	/// Gets the last value assigned to `self`, or returns an [`Error::UndefinedVariable`] if we
	/// haven't been assigned to yet.
	pub fn run(&self) -> Result<Value> {
		self.fetch().ok_or_else(|| Error::UndefinedVariable(self.name().to_boxed()))
	}
}

impl Debug for Variable {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Variable")
				.field("name", &self.name())
				.field("value", &(self.0).1.borrow())
				.finish()
		} else {
			write!(f, "Variable({})", self.name())
		}
	}
}
