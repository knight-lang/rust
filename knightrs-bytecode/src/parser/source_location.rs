use crate::parser::{ParseError, ParseErrorKind};
use std::fmt::{self, Display, Formatter};
use std::path::Path;

/// A location within a Knight program.
///
/// It's used both in parse error messages (indicating where an exception occurred), as well as
/// runtime errors (and when stacktraces are enabled, whole stacktraces are shown.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceLocation<'path> {
	source: ProgramSource<'path>,
	lineno: usize,
}

/// Whence a program originates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgramSource<'path> {
	/// The program originates from a file.
	File(&'path Path),
	/// The program originates from the `-e` arg given on the command line.
	ExprFlag,
	/// The program originates from somewhere else.
	Other(&'static str),

	/// The program originates from the `EVAL` extension
	#[cfg(feature = "extensions")]
	Eval, // todo: do we want to record where the eval came from?
}

impl<'path> SourceLocation<'path> {
	/// Creates a new [`SourceLocation`] for the the source and line number.
	///
	/// It's a logical error for `lineno` to be zero, as line numbering starts at one. However, this
	/// is only checked in debug mode as it's not a requirement for anything else.
	pub const fn new(source: ProgramSource<'path>, lineno: usize) -> Self {
		debug_assert!(lineno != 0);

		Self { source, lineno }
	}

	/// The filename of this source location.
	pub const fn source(&self) -> ProgramSource<'path> {
		self.source
	}

	/// The line number for this source location/
	pub const fn lineno(&self) -> usize {
		self.lineno
	}
}

impl Display for SourceLocation<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}:{}", self.source, self.lineno)
	}
}

impl Display for ProgramSource<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::File(path) => write!(f, "{}", path.display()),
			Self::ExprFlag => f.write_str("-e"),
			Self::Other(other) => f.write_str(other),

			#[cfg(feature = "extensions")]
			Self::Eval => f.write_str("<eval>"),
		}
	}
}
