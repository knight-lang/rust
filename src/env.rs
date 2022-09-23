use crate::value::text::Character;
use crate::value::Runnable;
use crate::variable::IllegalVariableName;
use crate::{Function, Integer, Result, Text, TextSlice, Value, Variable};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, Write};

mod builder;
mod options;
pub use builder::Builder;
pub use options::Options;

type Stdin<'e> = dyn BufRead + 'e + Send + Sync;
type Stdout<'e> = dyn Write + 'e + Send + Sync;
type System<'e, E> =
	dyn FnMut(&TextSlice<E>, Option<&TextSlice<E>>) -> Result<Text<E>> + 'e + Send + Sync;
type ReadFile<'e, E> = dyn FnMut(&TextSlice<E>) -> Result<Text<E>> + 'e + Send + Sync;

/// The environment hosts all relevant information for knight programs.
pub struct Environment<'e, E> {
	options: Options,
	// We use a `HashSet` because we want the variable to own its name, which a `HashMap`
	// wouldn't allow for. (or would have redundant allocations.)
	variables: HashSet<Variable<'e, E>>,
	stdin: Box<Stdin<'e>>,
	stdout: Box<Stdout<'e>>,
	rng: Box<StdRng>,
	system: Box<System<'e, E>>,
	read_file: Box<ReadFile<'e, E>>,

	functions: HashMap<Character<E>, Function<'e, E>>,
	extensions: HashMap<Text<E>, Function<'e, E>>,

	// A queue of things that'll be read from for `PROMPT` instead of stdin.
	prompt_lines: VecDeque<Text<E>>,

	// A queue of things that'll be read from for `` ` `` instead of stdin.
	system_results: VecDeque<Text<E>>,
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Environment: Send, Sync);

impl<E> Default for Environment<'_, E> {
	fn default() -> Self {
		Builder::default().build()
	}
}

impl<'e, E> Environment<'e, E> {
	/// Parses and executes `source` as knight code.
	pub fn play(&mut self, source: &TextSlice<E>) -> Result<Value<'e, E>> {
		crate::Parser::new(source, self).parse_program()?.run(self)
	}

	pub fn functions(&self) -> &HashMap<Character<E>, Function<'e, E>> {
		&self.functions
	}

	pub fn options(&self) -> &Options {
		&self.options
	}

	pub fn stdin(&mut self) -> &mut dyn BufRead {
		&mut self.stdin
	}

	pub fn stdout(&mut self) -> &mut dyn Write {
		&mut self.stdout
	}

	/// Fetches the variable corresponding to `name` in the environment, creating one if it's the
	/// first time that name has been requested
	pub fn lookup(
		&mut self,
		name: &TextSlice<E>,
	) -> std::result::Result<Variable<'e, E>, IllegalVariableName> {
		// OPTIMIZE: This does a double lookup, which isnt spectacular.
		if let Some(var) = self.variables.get(name) {
			return Ok(var.clone());
		}

		let variable = Variable::new(name.into(), &self.options)?;
		self.variables.insert(variable.clone());
		Ok(variable)
	}

	/// Gets a random `Integer`.
	pub fn random(&mut self) -> Integer {
		let rand = self.rng.gen::<i32>().abs();

		Integer::from(if self.options().compliance.restrict_rand { rand & 0x7fff } else { rand })
	}

	/// Seeds the random number generator.
	pub fn srand(&mut self, seed: Integer) {
		*self.rng = StdRng::seed_from_u64(i64::from(seed) as u64)
	}

	/// Executes `command` as a shell command, returning its result.
	pub fn run_command(
		&mut self,
		command: &TextSlice<E>,
		stdin: Option<&TextSlice<E>>,
	) -> Result<Text<E>> {
		(self.system)(command, stdin)
	}

	/// Gets the list of known extension functions.
	pub fn extensions(&self) -> &HashMap<Text<E>, Function<'e, E>> {
		&self.extensions
	}

	/// Gets a mutable list of known extension functions, so you can add to them.
	pub fn extensions_mut(&mut self) -> &mut HashMap<Text<E>, Function<'e, E>> {
		&mut self.extensions
	}

	pub fn add_to_prompt(&mut self, line: Text<E>) {
		for line in (&**line).split('\n') {
			self.prompt_lines.push_back(line.try_into().unwrap());
		}
	}

	pub fn get_next_prompt_line(&mut self) -> Option<Text<E>> {
		self.prompt_lines.pop_front()
	}

	pub fn add_to_system(&mut self, output: Text<E>) {
		self.system_results.push_back(output);
	}

	pub fn get_next_system_result(&mut self) -> Option<Text<E>> {
		self.system_results.pop_front()
	}

	pub fn read_file(&mut self, filename: &TextSlice<E>) -> crate::Result<Text<E>> {
		(self.read_file)(filename)
	}
}
