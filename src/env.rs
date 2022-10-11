#[cfg(feature = "extensions")]
use crate::function::ExtensionFunction;
use crate::parse::{ParseFn, Parser};
use crate::value::integer::IntType;
use crate::value::text::Encoding;
use crate::value::Runnable;
use crate::{Function, Integer, RefCount, Result, Text, TextSlice, Value};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashSet;

mod builder;
pub mod flags;
pub mod output;
pub mod prompt;
mod variable;
pub use builder::Builder;
pub use flags::Flags;
use output::Output;
use prompt::Prompt;
pub use variable::{IllegalVariableName, Variable};

#[cfg(feature = "extensions")]
type System<'e, E> =
	dyn FnMut(&TextSlice<E>, Option<&TextSlice<E>>, &Flags) -> Result<Text<E>> + 'e + Send + Sync;

#[cfg(feature = "extensions")]
type ReadFile<'e, E> = dyn FnMut(&TextSlice<E>, &Flags) -> Result<Text<E>> + 'e + Send + Sync;

/// The environment hosts all relevant information for knight programs.
pub struct Environment<'e, I, E> {
	flags: &'e Flags,
	variables: HashSet<Variable<'e, I, E>>,
	prompt: Prompt<'e, I, E>,
	output: Output<'e, I, E>,
	functions: HashSet<Function<'e, I, E>>,
	rng: StdRng,

	// Parsers are only modifiable when the `extensions` feature is enabled. Otherwise, the normal
	// set of parsers is loaded up.
	parsers: Vec<RefCount<dyn ParseFn<'e, I, E>>>,

	// A List of extension functions.
	extensions: HashSet<ExtensionFunction<'e, I, E>>,

	// A queue of things that'll be read from for `` ` `` instead of stdin.
	#[cfg(feature = "extensions")]
	system_results: std::collections::VecDeque<Text<E>>,

	#[cfg(feature = "extensions")]
	system: Box<System<'e, E>>,

	#[cfg(feature = "extensions")]
	read_file: Box<ReadFile<'e, E>>,
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Environment<'_, (), ()>: Send, Sync);

impl<I: IntType, E: Encoding> Default for Environment<'_, I, E> {
	/// Creates a new [`Environment`] with all the default configuration flags.
	fn default() -> Self {
		Self::builder(&flags::DEFAULT).build()
	}
}

impl<'e, I: IntType, E: Encoding> Environment<'e, I, E> {
	/// Creates a new [`Environment`] with the default configuration.
	pub fn new() -> Self {
		Self::default()
	}

	/// A shorthand function for creating [`Builder`]s.
	pub fn builder(flags: &'e Flags) -> Builder<'e, I, E> {
		Builder::new(flags)
	}

	/// Parses and executes `source` as knight code.
	pub fn play(&mut self, source: &TextSlice<E>) -> Result<Value<'e, I, E>> {
		Parser::new(source, self).parse_program()?.run(self)
	}
}

impl<'e, I, E> Environment<'e, I, E> {
	/// Gets the list of flags for `self`.
	#[must_use]
	pub fn flags(&self) -> &'e Flags {
		self.flags
	}

	/// Gets the list of currently defined functions for `self`.
	#[must_use]
	pub fn functions(&self) -> &HashSet<Function<'e, I, E>> {
		&self.functions
	}

	/// Gets the list of currently defined parsers for `self`.
	#[must_use]
	pub fn parsers(&self) -> &[RefCount<dyn ParseFn<'e, I, E>>] {
		&self.parsers
	}

	/// Gets the [`Prompt`] type, which handles reading lines from stdin.
	#[must_use]
	pub fn prompt(&mut self) -> &mut Prompt<'e, I, E> {
		&mut self.prompt
	}

	/// Read a line from the [`prompt`](Self::prompt).
	///
	/// This exits because you need to pass a reference to `self.flags`
	pub fn read_line(&mut self) -> Result<Option<Text<E>>>
	where
		I: IntType,
		E: Encoding,
	{
		self.prompt.read_line(self.flags)?.get(self)
	}

	/// Gets the [`Output`] type, which handles writing lines to stdout.
	#[must_use]
	pub fn output(&mut self) -> &mut Output<'e, I, E> {
		&mut self.output
	}

	/// Fetches the variable corresponding to `name`, creating one if it's the first time that name
	/// has been requested.
	pub fn lookup(
		&mut self,
		name: &TextSlice<E>,
	) -> std::result::Result<Variable<'e, I, E>, IllegalVariableName>
	where
		E: Encoding,
	{
		// OPTIMIZE: This does a double lookup, which isnt spectacular.
		if let Some(var) = self.variables.get(name) {
			return Ok(var.clone());
		}

		let variable = Variable::new(name.into(), self.flags())?;
		self.variables.insert(variable.clone());
		Ok(variable)
	}

	/// Gets a random [`Integer`].
	#[must_use]
	pub fn random(&mut self) -> Integer<I>
	where
		I: IntType,
	{
		Integer::random(&mut self.rng, self.flags)
	}
}

#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
impl<'e, I, E> Environment<'e, I, E> {
	/// Gets the list of known extension functions.
	#[must_use]
	pub fn extensions(&self) -> &HashSet<ExtensionFunction<'e, I, E>> {
		&self.extensions
	}

	/// Seeds the random number generator.
	pub fn srand(&mut self, seed: Integer<I>)
	where
		I: IntType,
	{
		self.rng = StdRng::seed_from_u64(i64::from(seed) as u64)
	}

	/// Executes `command` as a shell command, returning its result.
	pub fn run_command(
		&mut self,
		command: &TextSlice<E>,
		stdin: Option<&TextSlice<E>>,
	) -> Result<Text<E>> {
		(self.system)(command, stdin, self.flags)
	}

	/// Adds `output` as the next value to return from the system command.
	pub fn add_to_system(&mut self, output: Text<E>) {
		self.system_results.push_back(output);
	}

	/// Gets the next result from within system.
	#[must_use]
	pub fn get_next_system_result(&mut self) -> Option<Text<E>> {
		self.system_results.pop_front()
	}

	/// Reads the file located at `filename`, returning its contents.
	pub fn read_file(&mut self, filename: &TextSlice<E>) -> Result<Text<E>> {
		(self.read_file)(filename, self.flags)
	}
}

impl<I, E> Drop for Environment<'_, I, E> {
	fn drop(&mut self) {
		// You can assign a variable to itself, which means that it'll end up leaking memory. So,
		// we have to manually ensure that no variables reference others.
		for var in self.variables.iter() {
			var.assign(Value::Null);
		}
	}
}
