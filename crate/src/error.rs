use crate::parser::ParseError;
use crate::text::IllegalChar;
use crate::variable::IllegalVariableName;
use crate::SharedText;
use std::fmt::{self, Display, Formatter};
use std::io;

/// All possible errors that can occur during knight program execution.
#[derive(Debug)]
pub enum Error {
	/// Indicates that a conversion does not exist
	NoConversion { from: &'static str, to: &'static str },

	/// An undefined variable was accessed.
	UndefinedVariable(SharedText),

	/// There was a problem with I/O.
	IoError(io::Error),

	/// A type was given to a function that doesn't support it.
	TypeError(&'static str),

	/// The correct type was supplied, but some requirements for it weren't met.
	DomainError(&'static str),

	/// Division/Modulo by zero.
	DivisionByZero,

	/// There was an issue with parsing (eg `EVAL` failed).
	ParseError(ParseError),

	/// The `QUIT` command was run.
	Quit(i32),

	/// A variable name was illegal.
	IllegalVariableName(IllegalVariableName),

	/// An illegal character appeared in the source code.
	#[cfg(feature = "strict-charset")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "strict-charset")))]
	IllegalChar(IllegalChar),

	/// An integer operation overflowed. Only used when the `checked-overflow` feature is enabled.
	#[cfg(feature = "checked-overflow")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "checked-overflow")))]
	IntegerOverflow,

	#[cfg(feature = "out-of-bounds-errors")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "out-of-bounds-errors")))]
	IndexOutOfBounds { len: usize, index: usize },
}

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		Self::IoError(err)
	}
}

impl From<IllegalChar> for Error {
	fn from(err: IllegalChar) -> Self {
		#[cfg(feature = "strict-charset")]
		{
			Self::IllegalChar(err)
		}

		#[cfg(not(feature = "strict-charset"))]
		{
			todo!()
		}
	}
}

impl From<ParseError> for Error {
	fn from(err: ParseError) -> Self {
		Self::ParseError(err)
	}
}

impl From<IllegalVariableName> for Error {
	fn from(err: IllegalVariableName) -> Self {
		Self::IllegalVariableName(err)
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		match self {
			Self::ParseError(err) => Some(err),
			Self::IoError(err) => Some(err),
			Self::IllegalVariableName(err) => Some(err),
			#[cfg(feature = "strict-charset")]
			Self::IllegalChar(err) => Some(err),
			_ => None,
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::NoConversion { from, to } => write!(f, "undefined conversion from {from} to {to}"),
			Self::UndefinedVariable(name) => write!(f, "undefined variable {name} was accessed"),
			Self::IoError(err) => write!(f, "an io error occurred: {err}"),
			Self::DomainError(err) => write!(f, "an domain error occurred: {err}"),
			Self::TypeError(kind) => write!(f, "invalid type {kind} given"),
			Self::DivisionByZero => write!(f, "division/modulo by zero"),
			Self::ParseError(err) => Display::fmt(&err, f),
			Self::Quit(status) => write!(f, "quitting with status code {status}"),
			Self::IllegalVariableName(err) => Display::fmt(&err, f),

			#[cfg(feature = "strict-charset")]
			Self::IllegalChar(err) => Display::fmt(&err, f),

			#[cfg(feature = "checked-overflow")]
			Self::IntegerOverflow => write!(f, "integer under/overflow"),

			#[cfg(feature = "out-of-bounds-errors")]
			Self::IndexOutOfBounds { len, index } => {
				write!(f, "end index {index} is out of bounds for string of length {len}")
			}
		}
	}
}
