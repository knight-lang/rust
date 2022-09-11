use super::*;
use std::collections::HashMap;
use std::io;

/// The environment hosts all relevant information for knight programs.
pub struct Builder<'e> {
	stdin: Option<Box<Stdin<'e>>>,
	stdout: Option<Box<Stdout<'e>>>,
	functions: HashMap<char, &'e Function>,
	extensions: HashMap<Text, &'e Function>,

	#[cfg(feature = "system-function")]
	system: Option<Box<System<'e>>>,

	#[cfg(feature = "use-function")]
	read_file: Option<Box<ReadFile<'e>>>,
}

impl Default for Builder<'_> {
	fn default() -> Self {
		Self {
			stdin: None,
			stdout: None,
			functions: crate::function::default(),
			extensions: Default::default(),

			#[cfg(feature = "system-function")]
			system: None,

			#[cfg(feature = "use-function")]
			read_file: None,
		}
	}
}
/*
	#[cfg(feature = "extension-functions")]
	extensions: {
		#[allow(unused_mut)]
		let mut map = HashMap::<Text, &'static crate::Function>::default();

		#[cfg(feature = "xsrand-function")]
		map.insert("SRAND".try_into().unwrap(), &crate::function::SRAND);

		#[cfg(feature = "xreverse-function")]
		map.insert("REV".try_into().unwrap(), &crate::function::REVERSE);

		#[cfg(feature = "xrange-function")]
		map.insert("RANGE".try_into().unwrap(), &crate::function::RANGE);

		map
	},
*/

impl<'e> Builder<'e> {
	pub fn stdin<S: BufRead + Send + Sync + 'e>(&mut self, stdin: S) {
		self.stdin = Some(Box::new(stdin) as Box<_>);
	}

	pub fn stdout<S: Write + Send + Sync + 'e>(&mut self, stdout: S) {
		self.stdout = Some(Box::new(stdout) as Box<_>);
	}

	pub fn functions(&mut self) -> &mut HashMap<char, &'e Function> {
		&mut self.functions
	}

	pub fn extensions(&mut self) -> &mut HashMap<Text, &'e Function> {
		&mut self.extensions
	}

	#[cfg(feature = "system-function")]
	pub fn system<F>(&mut self, func: F)
	where
		F: FnMut(&TextSlice, Option<&TextSlice>) -> crate::Result<Text> + Send + Sync + 'e,
	{
		self.system = Some(Box::new(func) as Box<_>);
	}

	#[cfg(feature = "use-function")]
	pub fn read_file<F>(&mut self, func: F)
	where
		F: FnMut(&TextSlice) -> crate::Result<Text> + Send + Sync + 'e,
	{
		self.read_file = Some(Box::new(func) as Box<_>);
	}

	pub fn build(self) -> Environment<'e> {
		Environment {
			variables: HashSet::default(),
			stdin: self.stdin.unwrap_or_else(|| Box::new(io::BufReader::new(io::stdin()))),
			stdout: self.stdout.unwrap_or_else(|| Box::new(io::stdout())),

			#[cfg(feature = "system-function")]
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

			#[cfg(feature = "use-function")]
			read_file: self.read_file.unwrap_or_else(|| {
				Box::new(|filename| Ok(std::fs::read_to_string(&**filename)?.try_into()?))
			}),

			extensions: self.extensions,
			functions: self.functions,
			rng: Box::new(StdRng::from_entropy()),

			#[cfg(feature = "assign-to-prompt")]
			prompt_lines: Default::default(),

			#[cfg(feature = "assign-to-system")]
			system_results: Default::default(),
		}
	}
}
