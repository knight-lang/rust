#![allow(unused)]
use super::*;
use crate::Mutable;
use crate::{Features, Function};
use std::collections::VecDeque;

cfg_if! {
	if #[cfg(feature = "multithreaded")] {
		pub trait Threadsafe: Send + Sync {}
		impl<T: Send + Sync> Threadsafe for T{}
	} else {
		pub trait Threadsafe {}
		impl<T> Threadsafe for T{}
	}
}

pub trait Stdin: BufRead + Threadsafe {}
impl<T: BufRead + Threadsafe> Stdin for T {}

pub trait Stdout: Write + Threadsafe {}
impl<T: Write + Threadsafe> Stdout for T {}

pub trait System: FnMut(&Text) -> crate::Result<SharedText> + Threadsafe {}
impl<T: FnMut(&Text) -> crate::Result<SharedText> + Threadsafe> System for T {}

pub struct Builder<'s> {
	features: &'s Features,
	variables: HashSet<Variable>,
	stdin: Option<Box<dyn Stdin + 's>>,
	stdout: Option<Box<dyn Stdout + 's>>,
	system: Option<Box<dyn System + 's>>,
	rng: Option<Box<StdRng>>, // todo: maybe have this customizable?

	functions: HashMap<char, &'s Function>,
	extension_functions: HashMap<&'s Text, &'s Function>,
	read_file: Option<Box<dyn System + 's>>,
}

impl<'s> Builder<'s> {
	pub fn new(features: &'s Features) -> Self {
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

	pub fn set_stdout<T: Stdout + 's>(&mut self, stdout: T) {
		self.stdout = Some(Box::new(stdout));
	}

	pub fn set_stdin<T: Stdin + 's>(&mut self, stdin: T) {
		self.stdin = Some(Box::new(stdin));
	}

	pub fn set_system<T: System + 's>(&mut self, system: T) {
		self.system = Some(Box::new(system));
	}

	pub fn set_read_file<T: System + 's>(&mut self, read_file: T) {
		assert!(self.features.functions.r#use, "`read_file` set when `use_function` isnt");

		self.read_file = Some(Box::new(read_file));
	}

	pub fn declare_function(&mut self, func: &'s Function) -> Option<&'s Function> {
		let first_char = func.name.chars().next().expect("empty function name");

		if first_char == 'X' {
			self.extension_functions.insert(func.name, func)
		} else {
			self.functions.insert(first_char, func)
		}
	}

	pub fn build(self) -> Environment {
		todo!();
	}
}

pub fn foo() {
	let features = Default::default();
	let mut b = Builder::new(&features);
	b.set_stdout(std::io::stdout());
	b.set_stdin(std::io::BufReader::new(std::io::stdin()));

	let mut i = Vec::new();
	let mut o = Vec::new();
	let mut c = Builder::new(&features);

	c.set_stdout(&mut i);
	c.set_stdin(&*o);
}
// 	// note that non-predefined variables
// 	pub fn define_variable(&mut self, name: SharedText, value: Value) {
// 		let _ = (name, value);
// 		todo!()
// 	}

// 	pub fn stdin(&mut self, stdin: super::Stdin)

// 	stdin: Option<BufReader<Box<Stdin>>>,
// 	stdout: Option<Box<Stdout>>,
// 	system: Option<Box<SystemCommand>>,
// 	rng: Option<Box<StdRng>>,

// 	#[cfg(feature = "extension-functions")]
// 	extension_functions: Option<HashMap<SharedText, &'static Function>>,

// }

// // #[cfg(feature = "multithreaded")]
// // sa::assert_impl_all!(Environment: Send, Sync);

// // impl Default for Environment {
// // 	fn default() -> Self {
// // 		Self {
// // 			variables: HashSet::default(),
// // 			stdin: BufReader::new(Box::new(std::io::stdin())),
// // 			stdout: Box::new(std::io::stdout()),
// // 			system: Box::new(|cmd| {
// // 				use std::process::{Command, Stdio};

// // 				let output = Command::new("/bin/sh")
// // 					.arg("-c")
// // 					.arg(&**cmd)
// // 					.stdin(Stdio::inherit())
// // 					.output()
// // 					.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;

// // 				Ok(SharedText::try_from(output)?)
// // 			}),
// // 			rng: Box::new(StdRng::from_entropy()),

// // 			#[cfg(feature = "extension-functions")]
// // 			extension_functions: {
// // 				let mut map = HashMap::<SharedText, &'static crate::Function>::default();

// // 				#[cfg(feature = "srand-function")]
// // 				map.insert("SRAND".try_into().unwrap(), &crate::function::SRAND);

// // 				#[cfg(feature = "reverse-function")]
// // 				map.insert("REV".try_into().unwrap(), &crate::function::REVERSE);

// // 				map
// // 			},

// // 			#[cfg(feature = "assign-to-prompt")]
// // 			prompt_lines: Default::default(),

// // 			#[cfg(feature = "assign-to-system")]
// // 			system_results: Default::default(),

// // 			#[cfg(feature = "use-function")]
// // 			readfile: Box::new(|filename| Ok(std::fs::read_to_string(&**filename)?.try_into()?)),
// // 		}
// // 	}
// // }

// // impl Environment {
// // 	/// Parses and executes `source` as knight code.
// // 	pub fn play(&mut self, source: &Text) -> crate::Result<Value> {
// // 		crate::Parser::new(source).parse(self)?.run(self)
// // 	}

// // 	/// Fetches the variable corresponding to `name` in the environment, creating one if it's the
// // 	/// first time that name has been requested
// // 	pub fn lookup(&mut self, name: &Text) -> Result<Variable, IllegalVariableName> {
// // 		// OPTIMIZE: This does a double lookup, which isnt spectacular.
// // 		if let Some(var) = self.variables.get(name) {
// // 			return Ok(var.clone());
// // 		}

// // 		let variable = Variable::new(name.into())?;
// // 		self.variables.insert(variable.clone());
// // 		Ok(variable)
// // 	}

// // 	/// Executes `command` as a shell command, returning its result.
// // 	pub fn run_command(&mut self, command: &Text) -> crate::Result<SharedText> {
// // 		(self.system)(command)
// // 	}

// // 	/// Gets a random `Integer`.
// // 	pub fn random(&mut self) -> Integer {
// // 		let rand = self.rng.gen::<Integer>().abs();

// // 		if cfg!(feature = "strict-compliance") {
// // 			rand & 0x7fff
// // 		} else {
// // 			rand
// // 		}
// // 	}

// // 	/// Seeds the random number generator.
// // 	#[cfg(feature = "srand-function")]
// // 	#[cfg_attr(doc_cfg, doc(cfg(feature = "srand-function")))]
// // 	pub fn srand(&mut self, seed: Integer) {
// // 		*self.rng = StdRng::seed_from_u64(seed as u64)
// // 	}

// // 	/// Gets the list of known extension functions.
// // 	#[cfg(feature = "extension-functions")]
// // 	#[cfg_attr(doc_cfg, doc(cfg(feature = "extension-functions")))]
// // 	pub fn extension_functions(&self) -> &HashMap<SharedText, &'static crate::Function> {
// // 		&self.extension_functions
// // 	}

// // 	/// Gets a mutable list of known extension functions, so you can add to them.
// // 	#[cfg(feature = "extension-functions")]
// // 	#[cfg_attr(doc_cfg, doc(cfg(feature = "extension-functions")))]
// // 	pub fn extensions_mut(&mut self) -> &mut HashMap<SharedText, &'static crate::Function> {
// // 		&mut self.extension_functions
// // 	}

// // 	#[cfg(feature = "assign-to-prompt")]
// // 	pub fn add_to_prompt(&mut self, line: SharedText) {
// // 		if line.contains('\n') {
// // 			todo!("split on `\\n` for `line`");
// // 		}

// // 		self.prompt_lines.push_back(line);
// // 	}

// // 	#[cfg(feature = "assign-to-prompt")]
// // 	pub fn get_next_prompt_line(&mut self) -> Option<SharedText> {
// // 		self.prompt_lines.pop_front()
// // 	}

// // 	#[cfg(feature = "assign-to-system")]
// // 	pub fn add_to_system(&mut self, output: SharedText) {
// // 		self.system_results.push_back(output);
// // 	}

// // 	#[cfg(feature = "assign-to-system")]
// // 	pub fn get_next_system_result(&mut self) -> Option<SharedText> {
// // 		self.system_results.pop_front()
// // 	}

// // 	#[cfg(feature = "use-function")]
// // 	pub fn read_file(&mut self, filename: &Text) -> crate::Result<SharedText> {
// // 		(self.readfile)(filename)
// // 	}
// // }

// // impl Read for Environment {
// // 	#[inline]
// // 	fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
// // 		self.stdin.read(data)
// // 	}
// // }

// // impl BufRead for Environment {
// // 	#[inline]
// // 	fn fill_buf(&mut self) -> io::Result<&[u8]> {
// // 		self.stdin.fill_buf()
// // 	}

// // 	#[inline]
// // 	fn consume(&mut self, amnt: usize) {
// // 		self.stdin.consume(amnt);
// // 	}

// // 	#[inline]
// // 	fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
// // 		self.stdin.read_line(buf)
// // 	}
// // }

// // impl Write for Environment {
// // 	#[inline]
// // 	fn write(&mut self, data: &[u8]) -> io::Result<usize> {
// // 		self.stdout.write(data)
// // 	}

// // 	#[inline]
// // 	fn flush(&mut self) -> io::Result<()> {
// // 		self.stdout.flush()
// // 	}
// // }
