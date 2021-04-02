use super::{RunCommand, Environment};
use crate::{RcString, RuntimeError};
use std::io::{self, Write, Read};
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::convert::TryFrom;

/// A Builder for the [`Environment`] struct.
///
/// # Examples
/// ```rust
/// use std::io::{Read, Write, Cursor};
/// use knightrs::Environment;
///
/// let mut stdin = Cursor::new("hello, world!");
/// let mut stdout = Vec::new();
///
/// let mut env =
/// 	Environment::builder()
/// 		.capacity(100)
/// 		.stdin(&mut stdin)
/// 		.stdout(&mut stdout)
/// 		.disable_system()
/// 		.build();
///
/// let mut out = String::new();
/// env.read_to_string(&mut out).expect("reading from `Cursor`s cannot fail");
/// assert_eq!(out, "hello, world!");
///
/// write!(env, "knights go on quests").expect("writing to `Vec`s cannot fail.");
/// drop(env); // so it no longer has a mutable reference to stdout.
/// assert_eq!(stdout, b"knights go on quests");
/// ```
#[derive(Default)]
pub struct Builder<I, O> {
	capacity: Option<usize>,
	stdin: Option<I>,
	stdout: Option<O>,
	run_command: Option<Box<RunCommand>>,
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
		write!(f, "cannot run '`' when as it is disabled.")
	}
}

impl std::error::Error for NotEnabled {}

fn run_command_err(_: &str) -> Result<RcString, RuntimeError> {
	Err(RuntimeError::Custom(Box::new(NotEnabled)))
}

fn run_command_system(cmd: &str) -> Result<RcString, RuntimeError> {
	let output =
		std::process::Command::new("sh")
			.arg("-c")
			.arg(cmd)
			.output()
			.map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;

	RcString::try_from(output).map_err(From::from)
}

impl<I, O> Builder<I, O> {
	/// Creates a new, default [`Builder`].
	///
	/// Note that this is aliased via [`Environment::builder()`], which doesn't require importing this type.
	///
	/// # Examples
	/// ```rust
	/// let env =
	/// 	knightrs::environment::Builder::new()
	/// 		.capacity(100)
	/// 		.disable_system()
	/// 		.build();
	/// // use env
	/// # let _ = env;
	/// ```
	#[must_use = "creating a builder does nothing by itself."]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the initial starting capacity for the set of [`Variable`](crate::Variable)s.
	///
	/// If not set, an (unspecified) default capacity is used.
	///
	/// # Examples
	/// ```rust
	/// use knightrs::Environment;
	///
	/// let env =
	/// 	Environment::builder()
	/// 		.capacity(256)
	/// 		.build();
	///
	/// // do stuff with env...
	/// # let _ = env;
	/// ```
	#[must_use = "assigning a capacity does nothing without calling 'build'."]
	pub fn capacity(mut self, capacity: usize) -> Self {
		self.capacity = Some(capacity);
		self
	}

	/// Sets the stdin for the [`Environment`].
	///
	/// This defaults to the [stdin](io::stdin) of the process.
	///
	/// # Examples
	/// ```rust
	/// use knightrs::Environment;
	/// use std::io::{Read, Cursor};
	///
	/// let mut stdin = Cursor::new("Line1\nline2\nwhatever");
	/// let mut env =
	/// 	Environment::builder()
	/// 		.stdin(&mut stdin)
	/// 		.build();
	///
	/// // do stuff with env..
	/// let mut out = String::new();
	///
	/// env.read_to_string(&mut out)
	/// 	.expect("Reading from `Cursor`s cannot fail");
	///
	/// assert_eq!(out, "Line1\nline2\nwhatever");
	/// ```
	#[must_use = "assigning to stdin does nothing without calling 'build'."]
	pub fn stdin(mut self, stdin: I) -> Self {
		self.stdin = Some(stdin);
		self
	}

	/// Sets the stdout for the [`Environment`].
	///
	/// This defaults to the [stdout](io::stdout) of the process.
	///
	/// # Examples
	/// ```rust
	/// use knightrs::Environment;
	/// use std::io::Write;
	///
	/// let mut stdout = Vec::new();
	/// let mut env =
	/// 	Environment::builder()
	/// 		.stdout(&mut stdout)
	/// 		.build();
	///
	/// // do stuff with env..
	/// write!(env, "something, something else.")
	/// 	.expect("writing to `Vec`s cannot fail.");
	///
	/// drop(env); // As it has a mutable reference to `stdout`.
	///
	/// assert_eq!(stdout, b"something, something else.");
	/// ```
	#[must_use = "assigning to stdout does nothing without calling 'build'."]
	pub fn stdout(mut self, stdout: O) -> Self {
		self.stdout = Some(stdout);
		self
	}

	/// Explicitly sets what should happen when the ["system" (`` ` ``)](crate::function::system) function is called.
	///
	/// The default value is to simply send the command to `sh` (ie `"sh", "-c", "command"`)
	///
	/// # Examples
	/// ```rust
	/// use knightrs::{Environment, RcString, RuntimeError};
	/// use std::convert::TryFrom;
	/// 
	/// let mut env =
	/// 	Environment::builder()
	/// 		.run_command(|input| {
	/// 			RcString::try_from(format!("Hello, {}", input))
	///				.map_err(RuntimeError::from)
	/// 		})
	/// 		.build();
	///
	/// assert_eq!(env.run_command("world").unwrap().as_str(), "Hello, world");
	/// ```
	#[must_use = "assigning a 'run_command' does nothing without calling 'build'."]
	pub fn run_command(mut self, run_command: impl FnMut(&str) -> Result<RcString, RuntimeError> + 'static) -> Self {
		self.run_command = Some(Box::new(run_command));
		self
	}

	/// Disables the ["system" (`` ` ``)](crate::function::system) command entirely.
	///
	/// When this is enabled, all calls to [`` ` ``](crate::function::system) will return errors.
	#[must_use = "disabling the system command to does nothing without calling 'build'."]
	pub fn disable_system(self) -> Self {
		// SAFETY: We're getting a mutable reference to a ZST, so this is always safe.
		self.run_command(run_command_err)
	}

	/// Creates a new [`Environment`] with all the supplied options.
	///
	/// Any options that have not been explicitly set will have their default values used.
	///
	/// # Examples
	/// ```rust,no_run
	/// # use knightrs::Environment;
	/// # use std::io::{Read, Write};
	/// let mut env = Environment::new();
	///
	/// // Write to stdout.
	/// writeln!(env, "Hello, world!");
	///
	/// // Read from stdin.
	/// let mut str = String::new();
	/// env.read_to_string(&mut str).expect("cant read from stdin!");
	///
	/// // execute command
	/// println!("The stdout of `ls -al` is {}", env.run_command("ls -al").expect("`ls -al` failed"));
	///
	/// // create a variable
	/// let var = env.get("foobar");
	/// assert_eq!(var, env.get("foobar")); // both variables are the same.
	/// ```

	#[must_use = "Simply calling `build` does nothing on its own."]
	pub fn build(self) -> Environment<I, O> {
		// SAFETY: All of these `unsafe` blocks are simply mutable references to ZSTs, which is always safe.
		Environment {
			vars: HashSet::with_capacity(self.capacity.unwrap_or(2048)),
			stdin: self.stdin.unwrap(), //unwrap_or(unsafe { &mut STDIN }),
			stdout: self.stdout.unwrap(), //unwrap_or(unsafe { &mut STDOUT }),
			run_command: self.run_command.unwrap_or(Box::new(run_command_system)),
			functions: crate::function::get_default_functions()
		}
	}
}
