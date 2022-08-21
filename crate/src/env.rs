use crate::{Error, Integer, KnStr, Result, SharedStr, Value};
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
use std::collections::HashSet;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader, Read, Write};

#[cfg(feature = "extension-functions")]
use std::collections::HashMap;

cfg_if! {
	if #[cfg(feature="multithreaded")] {
		type SystemCommand = dyn FnMut(&KnStr) -> Result<SharedStr> + Send + Sync;
		type Stdin = dyn Read + Send + Sync;
		type Stdout = dyn Write + Send + Sync;
	} else {
		type SystemCommand = dyn FnMut(&KnStr) -> Result<SharedStr>;
		type Stdin = dyn Read;
		type Stdout = dyn Write;
	}
}

/// The environment hosts all relevant information for knight programs.
pub struct Environment {
	// We use a `HashSet` because we want the variable to own its name, which a `HashMap`
	// wouldn't allow for. (or would have redundant allocations.)
	variables: HashSet<Variable>,

	// All the known extension functions.
	#[cfg(feature = "extension-functions")]
	extensions: HashMap<SharedStr, &'static crate::Function>,

	stdin: BufReader<Box<Stdin>>,
	stdout: Box<Stdout>,
	system: Box<SystemCommand>,
	rng: Box<StdRng>,
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Environment: Send, Sync);

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
			rng: Box::new(StdRng::from_entropy()),
			#[cfg(feature = "extension-functions")]
			extensions: {
				let mut map = HashMap::<SharedStr, &'static crate::Function>::default();

				#[cfg(feature = "srand-function")]
				map.insert("SRAND".try_into().unwrap(), &crate::function::SRAND);

				map
			},
		}
	}
}

/// Maximum length a name can have; only used when `verify-variable-names` is enabled.
pub const MAX_NAME_LEN: usize = 65535;

/// Indicates that a a variable name was illegal.
///
/// This is only ever returned if the `verify-variable-names` is enabled.
#[derive(Debug, PartialEq, Eq)]
pub enum IllegalVariableName {
	/// The name was empty
	Empty,

	/// The name was too long.
	TooLong(usize),

	/// The name had an illegal character at the beginning.
	IllegalStartingChar(char),

	/// The name had an illegal character in the middle.
	IllegalBodyChar(char),
}

impl Display for IllegalVariableName {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Empty => write!(f, "empty variable name supplied"),
			Self::TooLong(count) => write!(f, "variable name was too long ({count} > {MAX_NAME_LEN})"),
			Self::IllegalStartingChar(chr) => write!(f, "variable names cannot start with {chr:?}"),
			Self::IllegalBodyChar(chr) => write!(f, "variable names cannot include with {chr:?}"),
		}
	}
}

impl std::error::Error for IllegalVariableName {}

#[cfg(feature = "verify-variable-names")]
fn verify_variable_name(name: &KnStr) -> std::result::Result<(), IllegalVariableName> {
	use crate::parser::{is_lower, is_numeric};

	if MAX_NAME_LEN < name.len() {
		return Err(IllegalVariableName::TooLong(name.len()));
	}

	let first = name.chars().next().ok_or(IllegalVariableName::Empty)?;
	if !is_lower(first) {
		return Err(IllegalVariableName::IllegalStartingChar(first));
	}

	if let Some(bad) = name.chars().find(|&c| !is_lower(c) && !is_numeric(c)) {
		return Err(IllegalVariableName::IllegalBodyChar(bad));
	}

	Ok(())
}

impl Environment {
	/// Fetches the variable corresponding to `name` in the environment, creating one if it's the
	/// first time that name has been requested
	pub fn lookup(&mut self, name: &KnStr) -> std::result::Result<Variable, IllegalVariableName> {
		#[cfg(feature = "verify-variable-names")]
		verify_variable_name(name)?;

		// OPTIMIZE: This does a double lookup, which isnt spectacular.
		if let Some(var) = self.variables.get(name) {
			return Ok(var.clone());
		}

		let variable = Variable((name.into(), None.into()).into());
		self.variables.insert(variable.clone());
		Ok(variable)
	}

	pub fn run_command(&mut self, command: &KnStr) -> Result<SharedStr> {
		(self.system)(command)
	}

	// This is here in case we want to add seeding
	pub fn random(&mut self) -> Integer {
		let rand = self.rng.gen::<Integer>().abs();

		if cfg!(feature = "strict-compliance") {
			rand & 0x7fff
		} else {
			rand
		}
	}

	#[cfg(feature = "srand-function")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "srand-function")))]
	pub fn srand(&mut self, seed: u64) {
		*self.rng = StdRng::seed_from_u64(seed)
	}

	pub fn play(&mut self, source: &KnStr) -> Result<Value> {
		crate::Parser::new(source).parse(self)?.run(self)
	}

	pub fn extensions(&self) -> &HashMap<SharedStr, &'static crate::Function> {
		&self.extensions
	}

	pub fn extensions_mut(&mut self) -> &mut HashMap<SharedStr, &'static crate::Function> {
		&mut self.extensions
	}
}

impl Read for Environment {
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

#[derive(Clone)]
#[rustfmt::skip]
pub struct Variable(crate::RefCount<(SharedStr, crate::Mutable<Option<Value>>)>);

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Variable: Send, Sync);

impl Debug for Variable {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Variable")
				.field("name", &self.name())
				.field("value", &self.fetch())
				.finish()
		} else {
			write!(f, "Variable({})", self.name())
		}
	}
}

impl std::borrow::Borrow<KnStr> for Variable {
	#[inline]
	fn borrow(&self) -> &KnStr {
		self.name()
	}
}

impl Eq for Variable {}
impl PartialEq for Variable {
	/// Checks to see if two variables are equal.
	///
	/// This'll just check to see if their names are equivalent. Techincally, this means that
	/// two variables with the same name, but derived from different [`Environment`]s will end up
	/// being the same
	#[inline]
	fn eq(&self, rhs: &Self) -> bool {
		self.name() == rhs.name()
	}
}

impl Hash for Variable {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name().hash(state);
	}
}

impl Variable {
	/// Fetches the name of the variable.
	#[must_use]
	#[inline]
	pub fn name(&self) -> &SharedStr {
		&(self.0).0
	}

	/// Assigns a new value to the variable, returning whatever the previous value was.
	pub fn assign(&self, new: Value) -> Option<Value> {
		#[cfg(feature = "multithreaded")]
		{
			(self.0).1.write().expect("rwlock poisoned").replace(new)
		}

		#[cfg(not(feature = "multithreaded"))]
		{
			(self.0).1.replace(Some(new))
		}
	}

	/// Fetches the last value assigned to `self`, returning `None` if we haven't been assigned to yet.
	#[must_use]
	pub fn fetch(&self) -> Option<Value> {
		#[cfg(feature = "multithreaded")]
		{
			(self.0).1.read().expect("rwlock poisoned").clone()
		}

		#[cfg(not(feature = "multithreaded"))]
		{
			(self.0).1.borrow().clone()
		}
	}

	/// Gets the last value assigned to `self`, or returns an [`Error::UndefinedVariable`] if we
	/// haven't been assigned to yet.
	pub fn run(&self) -> Result<Value> {
		self.fetch().ok_or_else(|| Error::UndefinedVariable(self.name().clone()))
	}
}
