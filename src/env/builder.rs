use super::*;

/// A Builder for an [`Environment`], allowing its different options to be configured.
#[must_use]
pub struct Builder<'e, I: IntType> {
	flags: Flags,
	prompt: Prompt<'e, I>,
	output: Output<'e, I>,
	functions: HashSet<Function<'e, I>>,
	parsers: Vec<RefCount<dyn ParseFn<'e, I>>>,

	#[cfg(feature = "extensions")]
	extensions: HashSet<ExtensionFunction<'e, I>>,

	#[cfg(feature = "extensions")]
	system: Option<Box<System<'e>>>,

	#[cfg(feature = "extensions")]
	read_file: Option<Box<ReadFile<'e>>>,
}

impl<I: IntType> Default for Builder<'_, I> {
	fn default() -> Self {
		Self::new(Flags::default())
	}
}

impl<'e, I: IntType> Builder<'e, I> {
	pub fn new(flags: Flags) -> Self {
		Self {
			flags,
			prompt: Prompt::default(),
			output: Output::default(),
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

	pub fn stdin<S: super::prompt::Stdin + 'e>(&mut self, stdin: S) {
		self.prompt.set_stdin(stdin);
	}

	pub fn stdout<S: super::output::Stdout + 'e>(&mut self, stdout: S) {
		self.output.set_stdout(stdout);
	}

	pub fn functions(&mut self) -> &mut HashSet<Function<'e, I>> {
		&mut self.functions
	}

	// We only allow access to the parsers when extensions are enabled.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn parsers(&mut self) -> &mut Vec<RefCount<dyn ParseFn<'e, I>>> {
		&mut self.parsers
	}

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn extensions(&mut self) -> &mut HashSet<ExtensionFunction<'e, I>> {
		&mut self.extensions
	}

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn system<F>(&mut self, func: F)
	where
		F: FnMut(&TextSlice, Option<&TextSlice>, &Flags) -> crate::Result<Text> + Send + Sync + 'e,
	{
		self.system = Some(Box::new(func) as Box<_>);
	}

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn read_file<F>(&mut self, func: F)
	where
		F: FnMut(&TextSlice, &Flags) -> crate::Result<Text> + Send + Sync + 'e,
	{
		self.read_file = Some(Box::new(func) as Box<_>);
	}

	pub fn build(self) -> Environment<'e, I> {
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
