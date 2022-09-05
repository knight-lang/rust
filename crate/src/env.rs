use crate::variable::IllegalVariableName;
use crate::{Function, Integer, Result, Text, TextSlice, Value, Variable};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashSet;
use std::io::{self, BufRead, Read, Write};

#[cfg(feature = "extension-functions")]
use std::collections::HashMap;

mod builder;
pub use builder::Builder;

/// The environment hosts all relevant information for knight programs.
pub struct Environment<'e> {
	// We use a `HashSet` because we want the variable to own its name, which a `HashMap`
	// wouldn't allow for. (or would have redundant allocations.)
	variables: HashSet<Variable<'e>>,
	stdin: Box<dyn BufRead + 'e>,
	stdout: Box<dyn Write + 'e>,
	rng: Box<StdRng>,

	functions: HashMap<char, &'e Function>,

	// All the known extension functions.
	//
	// FIXME: Maybe we should make functions refcounted or something?
	#[cfg(feature = "extension-functions")]
	extensions: HashMap<Text, &'e Function>,

	// A queue of things that'll be read from for `PROMPT` instead of stdin.
	#[cfg(feature = "assign-to-prompt")]
	prompt_lines: std::collections::VecDeque<Text>,

	// A queue of things that'll be read from for `` ` `` instead of stdin.
	#[cfg(feature = "assign-to-prompt")]
	system_results: std::collections::VecDeque<Text>,

	#[cfg(feature = "system-function")]
	system: Box<dyn FnMut(&TextSlice) -> Result<Text> + 'e>,

	#[cfg(feature = "use-function")]
	read_file: Box<dyn FnMut(&TextSlice) -> Result<Text> + 'e>,
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Environment: Send, Sync);

impl Default for Environment<'_> {
	fn default() -> Self {
		Builder::default().build()
	}
}

impl<'e> Environment<'e> {
	/// Parses and executes `source` as knight code.
	pub fn play(&mut self, source: &TextSlice) -> Result<Value<'e>> {
		crate::Parser::new(source).parse(self)?.run(self)
	}

	pub fn lookup_function(&self, name: char) -> Option<&'e Function> {
		self.functions.get(&name).copied()
	}

	/// Fetches the variable corresponding to `name` in the environment, creating one if it's the
	/// first time that name has been requested
	pub fn lookup(
		&mut self,
		name: &TextSlice,
	) -> std::result::Result<Variable<'e>, IllegalVariableName> {
		// OPTIMIZE: This does a double lookup, which isnt spectacular.
		if let Some(var) = self.variables.get(name) {
			return Ok(var.clone());
		}

		let variable = Variable::new(name.into())?;
		self.variables.insert(variable.clone());
		Ok(variable)
	}

	/// Gets a random `Integer`.
	pub fn random(&mut self) -> Integer {
		let rand = self.rng.gen::<i32>().abs();

		Integer::from(if cfg!(feature = "strict-compliance") { rand & 0x7fff } else { rand })
	}

	/// Seeds the random number generator.
	pub fn srand(&mut self, seed: Integer) {
		*self.rng = StdRng::seed_from_u64(i64::from(seed) as u64)
	}

	/// Executes `command` as a shell command, returning its result.
	#[cfg(feature = "system-function")]
	pub fn run_command(&mut self, command: &TextSlice) -> Result<Text> {
		(self.system)(command)
	}

	/// Gets the list of known extension functions.
	#[cfg(feature = "extension-functions")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "extension-functions")))]
	pub fn extensions(&self) -> &HashMap<Text, &'e Function> {
		&self.extensions
	}

	/// Gets a mutable list of known extension functions, so you can add to them.
	#[cfg(feature = "extension-functions")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "extension-functions")))]
	pub fn extensions_mut(&mut self) -> &mut HashMap<Text, &'e Function> {
		&mut self.extensions
	}

	#[cfg(feature = "assign-to-prompt")]
	pub fn add_to_prompt(&mut self, line: Text) {
		for line in (&**line).split('\n') {
			self.prompt_lines.push_back(line.try_into().unwrap());
		}
	}

	#[cfg(feature = "assign-to-prompt")]
	pub fn get_next_prompt_line(&mut self) -> Option<Text> {
		self.prompt_lines.pop_front()
	}

	#[cfg(feature = "assign-to-system")]
	pub fn add_to_system(&mut self, output: Text) {
		self.system_results.push_back(output);
	}

	#[cfg(feature = "assign-to-system")]
	pub fn get_next_system_result(&mut self) -> Option<Text> {
		self.system_results.pop_front()
	}

	#[cfg(feature = "use-function")]
	pub fn read_file(&mut self, filename: &TextSlice) -> crate::Result<Text> {
		(self.read_file)(filename)
	}
}

impl Read for Environment<'_> {
	#[inline]
	fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
		self.stdin.read(data)
	}
}

impl BufRead for Environment<'_> {
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

impl Write for Environment<'_> {
	#[inline]
	fn write(&mut self, data: &[u8]) -> io::Result<usize> {
		self.stdout.write(data)
	}

	#[inline]
	fn flush(&mut self) -> io::Result<()> {
		self.stdout.flush()
	}
}
