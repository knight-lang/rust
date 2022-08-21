use crate::variable::IllegalVariableName;
use crate::{Error, Integer, SharedText, Text, Value, Variable};
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
use std::collections::HashSet;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader, Read, Write};

#[cfg(feature = "extension-functions")]
use std::collections::HashMap;

cfg_if! {
	if #[cfg(feature="multithreaded")] {
		type SystemCommand = dyn FnMut(&Text) -> crate::Result<SharedText> + Send + Sync;
		type Stdin = dyn Read + Send + Sync;
		type Stdout = dyn Write + Send + Sync;
	} else {
		type SystemCommand = dyn FnMut(&Text) -> crate::Result<SharedText>;
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
	//
	// FIXME: Maybe we should make functions refcounted or something?
	#[cfg(feature = "extension-functions")]
	extensions: HashMap<SharedText, &'static crate::Function>,

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
			system: Box::new(|cmd: &Text| {
				use std::process::{Command, Stdio};

				let output = Command::new("/bin/sh")
					.arg("-c")
					.arg(&**cmd)
					.stdin(Stdio::inherit())
					.output()
					.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;

				Ok(SharedText::try_from(output)?)
			}),
			rng: Box::new(StdRng::from_entropy()),
			#[cfg(feature = "extension-functions")]
			extensions: {
				let mut map = HashMap::<SharedText, &'static crate::Function>::default();

				#[cfg(feature = "srand-function")]
				map.insert("SRAND".try_into().unwrap(), &crate::function::SRAND);

				map
			},
		}
	}
}

impl Environment {
	/// Fetches the variable corresponding to `name` in the environment, creating one if it's the
	/// first time that name has been requested
	pub fn lookup(&mut self, name: &Text) -> Result<Variable, IllegalVariableName> {
		// OPTIMIZE: This does a double lookup, which isnt spectacular.
		if let Some(var) = self.variables.get(name) {
			return Ok(var.clone());
		}

		let variable = Variable::new(name.into())?;
		self.variables.insert(variable.clone());
		Ok(variable)
	}

	/// Executes `command` as a shell command, returning its result.
	pub fn run_command(&mut self, command: &Text) -> crate::Result<SharedText> {
		(self.system)(command)
	}

	/// Gets a random `Integer`.
	pub fn random(&mut self) -> Integer {
		let rand = self.rng.gen::<Integer>().abs();

		if cfg!(feature = "strict-compliance") {
			rand & 0x7fff
		} else {
			rand
		}
	}

	/// Seeds the random number generator.
	#[cfg(feature = "srand-function")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "srand-function")))]
	pub fn srand(&mut self, seed: Integer) {
		*self.rng = StdRng::seed_from_u64(seed as u64)
	}

	/// Parses and executes `source` as knight code.
	pub fn play(&mut self, source: &Text) -> crate::Result<Value> {
		crate::Parser::new(source).parse(self)?.run(self)
	}

	/// Gets the list of known extension functions.
	#[cfg(feature = "extension-functions")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "extension-functions")))]
	pub fn extensions(&self) -> &HashMap<SharedText, &'static crate::Function> {
		&self.extensions
	}

	/// Gets a mutable list of known extension functions, so you can add to them.
	#[cfg(feature = "extension-functions")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "extension-functions")))]
	pub fn extensions_mut(&mut self) -> &mut HashMap<SharedText, &'static crate::Function> {
		&mut self.extensions
	}
}

impl Read for Environment {
	#[inline]
	fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
		self.stdin.read(data)
	}
}

impl BufRead for Environment {
	#[inline]
	fn fill_buf(&mut self) -> io::Result<&[u8]> {
		self.stdin.fill_buf()
	}

	#[inline]
	fn consume(&mut self, amnt: usize) {
		self.stdin.consume(amnt);
	}

	#[inline]
	fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
		self.stdin.read_line(buf)
	}
}

impl Write for Environment {
	#[inline]
	fn write(&mut self, data: &[u8]) -> io::Result<usize> {
		self.stdout.write(data)
	}

	#[inline]
	fn flush(&mut self) -> io::Result<()> {
		self.stdout.flush()
	}
}
