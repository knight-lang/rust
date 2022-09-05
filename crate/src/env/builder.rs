#![allow(unused)]

use super::*;
use crate::Result;
use std::collections::HashMap;
use std::io;

/// The environment hosts all relevant information for knight programs.
pub struct Builder<'a> {
	stdin: Option<Box<dyn BufRead + 'a>>,
	stdout: Option<Box<dyn Write + 'a>>,
	functions: HashMap<char, &'a Function>,
	extensions: HashMap<Text, &'a Function>,

	#[cfg(feature = "system-function")]
	system: Option<Box<dyn FnMut(&TextSlice) -> Result<Text> + 'a>>,

	#[cfg(feature = "use-function")]
	read_file: Option<Box<dyn FnMut(&TextSlice) -> Result<Text> + 'a>>,
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

impl<'a> Builder<'a> {
	pub fn stdin<S: BufRead + 'a>(&mut self, stdin: S) {
		self.stdin = Some(Box::new(stdin) as Box<dyn BufRead + 'a>);
	}

	pub fn stdout<S: Write + 'a>(&mut self, stdout: S) {
		self.stdout = Some(Box::new(stdout) as Box<dyn Write + 'a>);
	}

	pub fn functions(&mut self) -> &mut HashMap<char, &'a Function> {
		&mut self.functions
	}

	pub fn extensions(&mut self) -> &mut HashMap<Text, &'a Function> {
		&mut self.extensions
	}

	#[cfg(feature = "system-function")]
	pub fn system<F: FnMut(&TextSlice) -> Result<Text> + 'a>(&mut self, func: F) {
		self.system = Some(Box::new(func) as Box<dyn FnMut(&TextSlice) -> Result<Text> + 'a>);
	}

	#[cfg(feature = "use-function")]
	pub fn read_file<F: FnMut(&TextSlice) -> Result<Text> + 'a>(&mut self, func: F) {
		self.read_file = Some(Box::new(func) as Box<dyn FnMut(&TextSlice) -> Result<Text> + 'a>);
	}

	pub fn build(self) -> Environment<'a> {
		/// The environment hosts all relevant information for knight programs.
		Environment {
			variables: HashSet::default(),
			stdin: self.stdin.unwrap_or_else(Box::new(io::BufReader::new(io::stdin()))),
			stdout: self.stdout.unwrap_or_else(Box::new(io::stdout())),

			#[cfg(feature = "system-function")]
			system: self.system.unwrap_or_else(|| {
				use std::process::{Command, Stdio};

				let output = Command::new("/bin/sh")
					.arg("-c")
					.arg(&**cmd)
					.stdin(Stdio::inherit())
					.output()
					.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;

				Ok(Text::try_from(output)?)
			}),
			..Default::default() // // We use a `HashSet` because we want the variable to own its name, which a `HashMap`
			                     // // wouldn't allow for. (or would have redundant allocations.)
			                     // variables: HashSet<Variable>,
			                     // stdin: BufReader<Box<Stdin>>,
			                     // stdout: Box<Stdout>,
			                     // system: Box<SystemCommand>,
			                     // rng: Box<StdRng>,

			                     // functions: HashMap<char, &'static Function>,

			                     // // All the known extension functions.
			                     // //
			                     // // FIXME: Maybe we should make functions refcounted or something?
			                     // #[cfg(feature = "extension-functions")]
			                     // extensions: HashMap<Text, &'static crate::Function>,

			                     // // A queue of things that'll be read from for `PROMPT` instead of stdin.
			                     // #[cfg(feature = "assign-to-prompt")]
			                     // prompt_lines: std::collections::VecDeque<Text>,

			                     // // A queue of things that'll be read from for `` ` `` instead of stdin.
			                     // #[cfg(feature = "assign-to-prompt")]
			                     // system_results: std::collections::VecDeque<Text>,

			                     // // The function that governs reading a file.
			                     // #[cfg(feature = "use-function")]
			                     // readfile: Box<ReadFile>,
		}
	}
}

fn foo() {
	let mut i = Vec::new();
	let mut o = Vec::new();

	let mut builder = Builder::default();
	builder.stdin(&*i);
	builder.stdout(&mut o);
}
