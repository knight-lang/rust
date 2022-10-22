use super::*;

/// A builder for an [`Environment`], allowing different options to be configured.
#[must_use]
pub struct Builder<'e> {
	flags: &'e Flags,
	prompt: Prompt<'e>,
	output: Output<'e>,
	functions: HashSet<Function>,

	// While not feature gated to extensions, it's only modifiable with extensions.
	parsers: Vec<ParseFn>,

	#[cfg(feature = "extensions")]
	extensions: HashSet<ExtensionFunction>,

	#[cfg(feature = "extensions")]
	system: Option<Box<System<'e>>>,

	#[cfg(feature = "extensions")]
	read_file: Option<Box<ReadFile<'e>>>,
}

impl Default for Builder<'_> {
	/// Creates a new [`Builder`] with [default flags](Flags::default).
	fn default() -> Self {
		Self::new(&crate::env::flags::DEFAULT)
	}
}

impl<'e> Builder<'e> {
	/// Creates a new [`Builder`] with the given flags.
	pub fn new(flags: &'e Flags) -> Self {
		Self {
			flags,
			prompt: Prompt::new(flags),
			output: Output::new(flags),
			functions: Function::default_set(&flags),
			parsers: crate::parse::default(&flags),

			#[cfg(feature = "extensions")]
			extensions: ExtensionFunction::default_set(&flags),

			#[cfg(feature = "extensions")]
			system: None,

			#[cfg(feature = "extensions")]
			read_file: None,
		}
	}

	/// Sets the stdin, which is used when `PROMPT` is run.
	///
	/// # Examples
	pub fn stdin<S: super::prompt::Stdin + 'e>(&mut self, stdin: S) {
		self.prompt.set_stdin(stdin);
	}

	/// Sets the stdout, which is used when `OUTPUT` and `DUMP` are run.
	pub fn stdout<S: super::output::Stdout + 'e>(&mut self, stdout: S) {
		self.output.set_stdout(stdout);
	}

	/// Gets a mutable set of normal (i.e. non-`X`) functions.
	///
	/// See [`Builder::extensions`] for extension functions.
	pub fn functions(&mut self) -> &mut HashSet<Function> {
		&mut self.functions
	}

	/// Gets a list of extension (i.e. `X`) functions.
	///
	/// See [`Builder::functions`] for normal functions.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn extensions(&mut self) -> &mut HashSet<ExtensionFunction> {
		&mut self.extensions
	}

	/// Gets a list of parse functions, which can be used to modify how parsing is done.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn parse_fns(&mut self) -> &mut Vec<ParseFn> {
		&mut self.parsers
	}

	/// Configure what happens when `$` is run.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn system<F>(&mut self, func: F)
	where
		F: FnMut(&TextSlice, Option<&TextSlice>, &Flags) -> crate::Result<Text> + 'e + Send + Sync,
	{
		self.system = Some(Box::new(func) as Box<_>);
	}

	/// Configure what happens when `USE` is run.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn read_file<F>(&mut self, func: F)
	where
		F: FnMut(&TextSlice, &Flags) -> crate::Result<Text> + 'e + Send + Sync,
	{
		self.read_file = Some(Box::new(func) as Box<_>);
	}

	/// Finishes the builder and creates the given environment.
	///
	/// Any values not set use their default values.
	pub fn build(self) -> Environment<'e> {
		Environment {
			flags: self.flags,

			variables: HashSet::default(),
			prompt: self.prompt,
			output: self.output,
			functions: self.functions,
			parsers: self.parsers,

			rng: StdRng::from_entropy(),

			#[cfg(feature = "extensions")]
			extensions: self.extensions,

			#[cfg(feature = "extensions")]
			system: self.system.unwrap_or_else(|| {
				Box::new(|cmd, stdin, flags| {
					use std::process::{Command, Stdio};

					assert!(stdin.is_none(), "todo, system function with non-default stdin");

					let output = Command::new("/bin/sh")
						.arg("-c")
						.arg(&**cmd)
						.stdin(Stdio::inherit())
						.output()
						.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;

					Ok(Text::new(output, flags)?)
				})
			}),

			#[cfg(feature = "extensions")]
			read_file: self.read_file.unwrap_or_else(|| {
				Box::new(|filename, flags| Ok(Text::new(std::fs::read_to_string(&**filename)?, flags)?))
			}),

			#[cfg(feature = "extensions")]
			system_results: Default::default(),
		}
	}
}
