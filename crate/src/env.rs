use crate::{Error, KnStr, Result, SharedStr, Value};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::rc::Rc;

type SystemCommand = dyn FnMut(&KnStr) -> Result<SharedStr>;

pub struct Environment {
	// We use a `HashSet` because we want the variable to own its name, which a `HashMap`
	// wouldn't allow for. (or would have redundant allocations.)
	variables: HashSet<Variable>,
	stdin: BufReader<Box<dyn Read>>,
	stdout: Box<dyn Write>,
	system: Box<SystemCommand>,
}

impl Default for Environment {
	fn default() -> Self {
		Self {
			variables: HashSet::default(),
			stdin: BufReader::new(Box::new(std::io::stdin())),
			stdout: Box::new(std::io::stdout()),
			system: Box::new(|cmd: &KnStr| {
				use std::process::{Command, Stdio};

				let output = Command::new("/bin/sh")
					.arg("-c")
					.arg(&**cmd)
					.stdin(Stdio::inherit())
					.output()
					.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;

				Ok(SharedStr::try_from(output)?)
			}),
		}
	}
}

impl Environment {
	/// Fetches the variable corresponding to `name` in the environment, creating one if it's the
	/// first time that name has been requested
	pub fn lookup(&mut self, name: &KnStr) -> Variable {
		if let Some(var) = self.variables.get(name) {
			return var.clone();
		}

		let variable = Variable(Rc::new((name.to_boxed(), RefCell::new(None))));
		self.variables.insert(variable.clone());
		variable
	}

	pub fn play(&mut self, input: &KnStr) -> Result<Value> {
		crate::parser::Parser::new(input).parse_program(self)?.run(self)
	}

	pub fn run_command(&mut self, command: &KnStr) -> Result<SharedStr> {
		(self.system)(command)
	}

	// this is here in case we want to add seeding
	pub fn random(&mut self) -> crate::Number {
		rand::random()
	}
}

impl Read for Environment {
	/// Read bytes into `data` from `self`'s `stdin`.
	///
	/// The `stdin` can be customized at creation via [`Builder::stdin`].
	fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
		self.stdin.read(data)
	}
}

impl BufRead for Environment {
	fn fill_buf(&mut self) -> io::Result<&[u8]> {
		self.stdin.fill_buf()
	}

	fn consume(&mut self, amnt: usize) {
		self.stdin.consume(amnt);
	}

	fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
		self.stdin.read_line(buf)
	}
}

impl Write for Environment {
	fn write(&mut self, data: &[u8]) -> io::Result<usize> {
		self.stdout.write(data)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.stdout.flush()
	}
}

impl std::borrow::Borrow<KnStr> for Variable {
	fn borrow(&self) -> &KnStr {
		self.name()
	}
}

#[derive(Clone)]
pub struct Variable(Rc<(Box<KnStr>, RefCell<Option<Value>>)>);

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
	pub fn name(&self) -> &KnStr {
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
