#![allow(unused)]
use super::*;
use crate::Mutable;
use crate::{Features, Function};
use std::collections::VecDeque;

pub struct Builder<'f> {
	features: &'f Features,
	variables: HashSet<Variable<'f>>,
	stdin: Option<Box<dyn Stdin + 'f>>,
	stdout: Option<Box<dyn Stdout + 'f>>,
	system: Option<Box<dyn System + 'f>>,
	rng: Option<Box<StdRng>>, // todo: maybe have this customizable?

	functions: HashMap<char, &'f Function>,
	extension_functions: HashMap<&'f Text, &'f Function>,
	read_file: Option<Box<dyn System + 'f>>,
}

impl<'f> Builder<'f> {
	pub fn new(features: &'f Features) -> Self {
		let mut builder = Self {
			variables: HashSet::default(),
			functions: crate::function::default(),
			extension_functions: HashMap::default(),
			features,
			stdin: None,
			stdout: None,
			system: None,
			rng: None,
			read_file: None,
		};

		features.populate_functions(&mut builder);
		builder
	}

	pub fn set_stdout<T: Stdout + 'f>(&mut self, stdout: T) {
		self.stdout = Some(Box::new(stdout));
	}

	pub fn set_stdin<T: Stdin + 'f>(&mut self, stdin: T) {
		self.stdin = Some(Box::new(stdin));
	}

	pub fn set_system<T: System + 'f>(&mut self, system: T) {
		self.system = Some(Box::new(system));
	}

	pub fn set_read_file<T: System + 'f>(&mut self, read_file: T) {
		assert!(self.features.functions.r#use, "`read_file` set when `use_function` isnt");

		self.read_file = Some(Box::new(read_file));
	}

	pub fn declare_function(&mut self, func: &'f Function) -> Option<&'f Function> {
		let first_char = func.name.chars().next().expect("empty function name");

		if first_char == 'X' {
			self.extension_functions.insert(func.name, func)
		} else {
			self.functions.insert(first_char, func)
		}
	}

	pub fn build(self) -> Environment<'f> {
		fn default_stdin() -> Box<dyn Stdin + 'static> {
			Box::new(std::io::BufReader::new(std::io::stdin()))
		}

		fn default_stdout() -> Box<dyn Stdout + 'static> {
			Box::new(std::io::stdout())
		}

		fn default_system() -> Box<dyn System + 'static> {
			use std::process::{Command, Stdio};

			Box::new(|cmd| {
				Ok(Command::new("/bin/sh")
					.arg("-c")
					.arg(&**cmd)
					.stdin(Stdio::inherit())
					.output()
					.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?
					.try_into()?)
			})
		}

		fn default_readfile() -> Box<dyn System + 'static> {
			Box::new(|filename| Ok(std::fs::read_to_string(&**filename)?.try_into()?))
		}

		Environment {
			features: self.features,
			variables: self.variables,
			stdin: if let Some(x) = self.stdin { x } else { default_stdin() },
			stdout: if let Some(x) = self.stdout { x } else { default_stdout() },
			system: if let Some(x) = self.system { x } else { default_system() },
			readfile: if let Some(x) = self.read_file { x } else { default_readfile() },
			rng: self.rng.unwrap_or_else(|| Box::new(StdRng::from_entropy())),

			functions: self.functions,
			extensions: self.extension_functions,

			prompt_lines: Default::default(),
			system_results: Default::default(),
		}
	}
}
