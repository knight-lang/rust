use super::Environment;
use crate::{Text, Result, Error};
use std::io::{self, Write, Read, BufReader};
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::convert::TryFrom;

/// A Builder for the [`Environment`] struct.
#[derive(Default)]
pub struct Builder<'i, 'o, 'c> {
	capacity: Option<usize>,
	stdin: Option<&'i mut dyn Read>,
	stdout: Option<&'o mut dyn Write>,
	system: Option<&'c mut dyn FnMut(&str) -> Result<Text>>,
}

// We have a lot of private ZST structs here and `static mut`s. This is because we need to have a mutable reference to,
// eg, a `dyn read`, but `io::stdin()` will return a new object. Thus, we simply make a ZST that calls `io::stdin()`
// every time it needs to read something.

struct Stdin;
static mut STDIN: Stdin = Stdin;

impl Read for Stdin {
	fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
		io::stdin().read(data)
	}
}

struct Stdout;
static mut STDOUT: Stdout = Stdout;

impl Write for Stdout {
	fn write(&mut self, data: &[u8]) -> io::Result<usize> {
		io::stdout().write(data)
	}

	fn flush(&mut self) -> io::Result<()> {
		io::stdout().flush()
	}
}

#[derive(Debug)]
struct NotEnabled;

impl Display for NotEnabled {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "cannot run '`' as it is disabled.")
	}
}

impl std::error::Error for NotEnabled {}

fn system_err(_: &str) -> Result<Text> {
	Err(Error::Custom(Box::new(NotEnabled)))
}

fn system_normal(cmd: &str) -> Result<Text> {
	use std::process::{Command, Stdio};

	let output =
		Command::new("/bin/sh")
			.arg("-c")
			.arg(cmd)
			.stdin(Stdio::inherit())
			.output()
			.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;

	Text::try_from(output).map_err(From::from)
}

static mut SYSTEM_ERR: fn(&str) -> Result<Text> = system_err;
static mut SYSTEM_NORMAL: fn(&str) -> Result<Text> = system_normal;

impl<'i, 'o, 'c> Builder<'i, 'o, 'c> {
	/// Creates a new, default [`Builder`].
	#[must_use = "creating a builder does nothing by itself."]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the initial starting capacity for the set of [`Variable`](crate::Variable)s.
	///
	/// If not set, an (unspecified) default capacity is used.
	#[must_use = "assigning a capacity does nothing without calling 'build'."]
	pub fn capacity(mut self, capacity: usize) -> Self {
		self.capacity = Some(capacity);
		self
	}

	/// Sets the stdin for the [`Environment`].
	///
	/// This defaults to the [stdin](io::stdin) of the process.
	#[must_use = "assigning to stdin does nothing without calling 'build'."]
	pub fn stdin(mut self, stdin: &'i mut dyn Read) -> Self {
		self.stdin = Some(stdin);
		self
	}

	/// Sets the stdout for the [`Environment`].
	///
	/// This defaults to the [stdout](io::stdout) of the process.
	#[must_use = "assigning to stdout does nothing without calling 'build'."]
	pub fn stdout(mut self, stdout: &'o mut dyn Write) -> Self {
		self.stdout = Some(stdout);
		self
	}

	/// Explicitly sets what should happen when the ["system" (`` ` ``)](crate::function::system) function is called.
	///
	/// The default value is to simply send the command to `sh` (ie `"sh", "-c", "command"`)
	#[must_use = "assigning a 'system' does nothing without calling 'build'."]
	pub fn system(mut self, system: &'c mut dyn FnMut(&str) -> Result<Text>) -> Self {
		self.system = Some(system);
		self
	}

	/// Disables the ["system" (`` ` ``)](crate::function::system) command entirely.
	///
	/// When this is enabled, all calls to [`` ` ``](crate::function::system) will return errors.
	#[must_use = "disabling the system command to does nothing without calling 'build'."]
	pub fn disable_system(self) -> Self {
		// SAFETY: We're getting a mutable reference to a ZST, so this is always safe.
		self.system(unsafe { &mut SYSTEM_ERR })
	}

	/// Creates a new [`Environment`] with all the supplied options.
	///
	/// Any options that have not been explicitly set will have their default values used.
	#[must_use = "Simply calling `build` does nothing on its own."]
	pub fn build(self) -> Environment<'i, 'o, 'c> {
		// SAFETY: All of these `unsafe` blocks are simply mutable references to ZSTs, which is always safe.
		Environment {
			vars: HashSet::with_capacity(self.capacity.unwrap_or(2048)),
			stdin: BufReader::new(self.stdin.unwrap_or(unsafe { &mut STDIN })),
			stdout: self.stdout.unwrap_or(unsafe { &mut STDOUT }),
			system: self.system.unwrap_or(unsafe { &mut SYSTEM_NORMAL })
		}
	}
}
