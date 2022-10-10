use super::*;
use std::collections::HashMap;

/// A Builder for an [`Environment`], allowing its different options to be configured.
#[must_use]
pub struct Builder<'e> {
	flags: Flags,
	prompt: Prompt<'e>,
	output: Output<'e>,
	functions: HashMap<Character, &'e Function>,

	#[cfg(feature = "extensions")]
	extensions: HashMap<Text, &'e Function>,

	#[cfg(feature = "extensions")]
	system: Option<Box<System<'e>>>,

	#[cfg(feature = "extensions")]
	read_file: Option<Box<ReadFile<'e>>>,
}

impl Default for Builder<'_> {
	fn default() -> Self {
		Self::new(Flags::default())
	}
}

impl<'e> Builder<'e> {
	pub fn new(flags: Flags) -> Self {
		Self {
			flags,
			prompt: Prompt::default(),
			output: Output::default(),
			functions: crate::function::default(&flags),

			#[cfg(feature = "extensions")]
			extensions: crate::function::extensions(&flags),

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

	pub fn functions(&mut self) -> &mut HashMap<Character, &'e Function> {
		&mut self.functions
	}

	pub fn build(self) -> Environment<'e> {
		Environment {
			flags: self.flags,

			variables: HashSet::default(),
			prompt: self.prompt,
			output: self.output,
			functions: self.functions,
			rng: StdRng::from_entropy(),

			#[cfg(feature = "extensions")]
			extensions: self.extensions,

			#[cfg(feature = "extensions")]
			system: self.system.unwrap_or_else(|| {
				Box::new(|cmd, stdin| {
					use std::process::{Command, Stdio};

					assert!(stdin.is_none(), "todo, system function with non-default stdin");

					let output = Command::new("/bin/sh")
						.arg("-c")
						.arg(&**cmd)
						.stdin(Stdio::inherit())
						.output()
						.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;

					Ok(Text::try_from(output)?)
				})
			}),

			#[cfg(feature = "extensions")]
			read_file: self.read_file.unwrap_or_else(|| {
				Box::new(|filename| Ok(std::fs::read_to_string(&**filename)?.try_into()?))
			}),

			#[cfg(feature = "extensions")]
			system_results: Default::default(),
		}
	}
}

#[cfg(feature = "extensions")]
impl<'e> Builder<'e> {
	pub fn extensions(&mut self) -> &mut HashMap<Text, &'e Function> {
		&mut self.extensions
	}

	pub fn system<F>(&mut self, func: F)
	where
		F: FnMut(&TextSlice, Option<&TextSlice>) -> crate::Result<Text> + Send + Sync + 'e,
	{
		self.system = Some(Box::new(func) as Box<_>);
	}

	pub fn read_file<F>(&mut self, func: F)
	where
		F: FnMut(&TextSlice) -> crate::Result<Text> + Send + Sync + 'e,
	{
		self.read_file = Some(Box::new(func) as Box<_>);
	}
}
