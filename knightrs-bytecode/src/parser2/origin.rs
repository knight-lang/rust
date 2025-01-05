use std::fmt::{self, Display, Formatter};
use std::path::Path;

/// Whence a program originates; used in backtrace error messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin<'src> {
	/// The program originates from a file.
	File(&'src Path),

	/// The program originates from the `-e` arg given on the command line.
	ExprFlag,

	/// The program originates from somewhere else.
	Other(&'static str),

	/// The program originates from the `EVAL` extension
	#[cfg(feature = "extensions")]
	Eval, // todo: do we want to record where the eval came from?
}

impl Display for Origin<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::File(path) => Display::fmt(&path.display(), f),
			Self::ExprFlag => f.write_str("-e"),
			Self::Other(other) => f.write_str(other),

			#[cfg(feature = "extensions")]
			Self::Eval => f.write_str("<eval>"),
		}
	}
}
