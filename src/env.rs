use crate::parse::{ParseFn, Parser};
use crate::value::Runnable;
#[cfg(feature = "extensions")]
use crate::{function::ExtensionFunction, value::Text};
use crate::{Function, Integer, RefCount, Result, TextSlice, Value};
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
type System<'e> = dyn FnMut(&TextSlice, Option<&TextSlice>) -> Result<Text> + 'e + Send + Sync;

#[cfg(feature = "extensions")]
type ReadFile<'e> = dyn FnMut(&TextSlice) -> Result<Text> + 'e + Send + Sync;

/// The environment hosts all relevant information for knight programs.
pub struct Environment<'e> {
	flags: Flags,
	variables: HashSet<Variable<'e>>,
	prompt: Prompt<'e>,
	output: Output<'e>,
	functions: HashSet<&'e Function<'e>>,
	rng: StdRng,

	// Parsers are only modifiable when the `extensions` feature is enabled. Otherwise, the normal
	// set of parsers is loaded up.
	parsers: Vec<RefCount<dyn ParseFn<'e>>>,

	// A List of extension functions.
	#[cfg(feature = "extensions")]
	extensions: HashSet<&'e ExtensionFunction<'e>>,

	// A queue of things that'll be read from for `` ` `` instead of stdin.
	#[cfg(feature = "extensions")]
	system_results: std::collections::VecDeque<Text>,

	#[cfg(feature = "extensions")]
	system: Box<System<'e>>,

	#[cfg(feature = "extensions")]
	read_file: Box<ReadFile<'e>>,
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Environment<'_>: Send, Sync);

impl Default for Environment<'_> {
	/// Creates a new [`Environment`] with all the default configuration flags.
	fn default() -> Self {
		Self::builder(Flags::default()).build()
	}
}

impl<'e> Environment<'e> {
	/// A shorthand function for creating [`Builder`]s.
	#[must_use]
	pub fn builder(flags: Flags) -> Builder<'e> {
		Builder::new(flags)
	}

	/// Parses and executes `source` as knight code.
	pub fn play(&mut self, source: &TextSlice) -> Result<Value<'e>> {
		Parser::new(source, self).parse_program()?.run(self)
	}

	/// Gets the list of flags for `self`.
	#[must_use]
	pub fn flags(&self) -> &Flags {
		&self.flags
	}

	/// Gets the list of currently defined functions for `self`.
	#[must_use]
	pub fn functions(&self) -> &HashSet<&'e Function<'e>> {
		&self.functions
	}

	/// Gets the list of currently defined parsers for `self`.
	#[must_use]
	pub fn parsers(&self) -> &[RefCount<dyn ParseFn<'e>>] {
		&self.parsers
	}

	/// Gets the [`Prompt`] type, which handles reading lines from stdin.
	#[must_use]
	pub fn prompt(&mut self) -> &mut Prompt<'e> {
		&mut self.prompt
	}

	/// Gets the [`Output`] type, which handles writing lines to stdout.
	#[must_use]
	pub fn output(&mut self) -> &mut Output<'e> {
		&mut self.output
	}

	/// Fetches the variable corresponding to `name`, creating one if it's the first time that name
	/// has been requested.
	pub fn lookup(
		&mut self,
		name: &TextSlice,
	) -> std::result::Result<Variable<'e>, IllegalVariableName> {
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
	pub fn random(&mut self) -> Integer {
		Integer::random(&mut self.rng, &self.flags)
	}
}

#[cfg(feature = "extensions")]
#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
impl<'e> Environment<'e> {
	/// Gets the list of known extension functions.
	#[must_use]
	pub fn extensions(&self) -> &HashSet<&'e ExtensionFunction<'e>> {
		&self.extensions
	}

	/// Seeds the random number generator.
	pub fn srand(&mut self, seed: Integer) {
		self.rng = StdRng::seed_from_u64(i64::from(seed) as u64)
	}

	/// Executes `command` as a shell command, returning its result.
	pub fn run_command(&mut self, command: &TextSlice, stdin: Option<&TextSlice>) -> Result<Text> {
		(self.system)(command, stdin)
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
		(self.read_file)(filename)
	}
}
