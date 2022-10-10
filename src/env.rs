#[cfg(feature = "extensions")]
use crate::function::ExtensionFunction;
use crate::parse::{ParseFn, Parser};
use crate::value::integer::IntType;
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
type System<'e> =
	dyn FnMut(&TextSlice, Option<&TextSlice>, &Flags) -> Result<Text> + 'e + Send + Sync;

#[cfg(feature = "extensions")]
type ReadFile<'e> = dyn FnMut(&TextSlice, &Flags) -> Result<Text> + 'e + Send + Sync;

/// The environment hosts all relevant information for knight programs.
pub struct Environment<'e, I: IntType> {
	flags: Flags,
	variables: HashSet<Variable<'e, I>>,
	prompt: Prompt<'e, I>,
	output: Output<'e, I>,
	functions: HashSet<Function<'e, I>>,
	rng: StdRng,

	// Parsers are only modifiable when the `extensions` feature is enabled. Otherwise, the normal
	// set of parsers is loaded up.
	parsers: Vec<RefCount<dyn ParseFn<'e, I>>>,

	// A List of extension functions.
	#[cfg(feature = "extensions")]
	extensions: HashSet<ExtensionFunction<'e, I>>,

	// A queue of things that'll be read from for `` ` `` instead of stdin.
	#[cfg(feature = "extensions")]
	system_results: std::collections::VecDeque<Text>,

	#[cfg(feature = "extensions")]
	system: Box<System<'e>>,

	#[cfg(feature = "extensions")]
	read_file: Box<ReadFile<'e>>,
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Environment<'_, I>: Send, Sync);

impl<I: IntType> Default for Environment<'_, I> {
	/// Creates a new [`Environment`] with all the default configuration flags.
	fn default() -> Self {
		Self::builder(Flags::default()).build()
	}
}

impl<'e, I: IntType> Environment<'e, I> {
	/// A shorthand function for creating [`Builder`]s.
	pub fn builder(flags: Flags) -> Builder<'e, I> {
		Builder::new(flags)
	}

	/// Parses and executes `source` as knight code.
	pub fn play(&mut self, source: &TextSlice) -> Result<Value<'e, I>> {
		Parser::new(source, self).parse_program()?.run(self)
	}

	/// Gets the list of flags for `self`.
	#[must_use]
	pub fn flags(&self) -> &Flags {
		&self.flags
	}

	/// Gets the list of currently defined functions for `self`.
	#[must_use]
	pub fn functions(&self) -> &HashSet<Function<'e, I>> {
		&self.functions
	}

	/// Gets the list of currently defined parsers for `self`.
	#[must_use]
	pub fn parsers(&self) -> &[RefCount<dyn ParseFn<'e, I>>] {
		&self.parsers
	}

	/// Gets the [`Prompt`] type, which handles reading lines from stdin.
	#[must_use]
	pub fn prompt(&mut self) -> &mut Prompt<'e, I> {
		&mut self.prompt
	}

	pub fn read_line(&mut self) -> Result<Option<Text>> {
		self.prompt.read_line(&self.flags)?.get(self)
	}

	/// Gets the [`Output`] type, which handles writing lines to stdout.
	#[must_use]
	pub fn output(&mut self) -> &mut Output<'e, I> {
		&mut self.output
	}

	/// Fetches the variable corresponding to `name`, creating one if it's the first time that name
	/// has been requested.
	pub fn lookup(
		&mut self,
		name: &TextSlice,
	) -> std::result::Result<Variable<'e, I>, IllegalVariableName> {
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
	pub fn random(&mut self) -> Integer<I> {
		Integer::random(&mut self.rng, &self.flags)
	}
}

#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
impl<'e, I: IntType> Environment<'e, I> {
	/// Gets the list of known extension functions.
	#[must_use]
	pub fn extensions(&self) -> &HashSet<ExtensionFunction<'e, I>> {
		&self.extensions
	}

	/// Seeds the random number generator.
	pub fn srand(&mut self, seed: Integer<I>) {
		self.rng = StdRng::seed_from_u64(i64::from(seed) as u64)
	}

	/// Executes `command` as a shell command, returning its result.
	pub fn run_command(&mut self, command: &TextSlice, stdin: Option<&TextSlice>) -> Result<Text> {
		(self.system)(command, stdin, &self.flags)
	}

	/// Adds `output` as the next value to return from the system command.
	pub fn add_to_system(&mut self, output: Text) {
		self.system_results.push_back(output);
	}

	/// Gets the next result from within system.
	#[must_use]
	pub fn get_next_system_result(&mut self) -> Option<Text> {
		self.system_results.pop_front()
	}

	/// Reads the file located at `filename`, returning its contents.
	pub fn read_file(&mut self, filename: &TextSlice) -> Result<Text> {
		(self.read_file)(filename, &self.flags)
	}
}
