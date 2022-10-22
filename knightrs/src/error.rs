use crate::env::variable::IllegalVariableName;
use crate::parse::Error as ParseError;
use crate::value::text::NewTextError;
use std::fmt::{self, Display, Formatter};
use std::io;

/// All possible errors that can occur during knight program execution.
// TODO: maybe include function name somewhere?
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	/// Indicates that a conversion does not exist
	NoConversion { from: &'static str, to: &'static str },

	/// An undefined variable was accessed.
	UndefinedVariable(String),

	/// There was a problem with I/O.
	IoError(io::Error),

	/// A type was given to a function that doesn't support it.
	TypeError(&'static str, &'static str),

	/// The correct type was supplied, but some requirements for it weren't met.
	DomainError(&'static str),

	/// Division/Modulo by zero.
	DivisionByZero,

	/// There was an issue with parsing
	///
	/// This is normally returned by the [`Parser`](crate::parse::Parser), but the [`EVAL`](
	/// crate::function::EVAL) extension can also cause this.
	ParseError(ParseError),

	/// The `QUIT` command was run.
	///
	/// Instead of actually exiting the process (which would make Knight unable to be embedded), this
	/// error is returned; the caller can do what they wish then.
	Quit(i32),

	/// Indicates that either `GET` or `SET` were given an index that was out of bounds.
	IndexOutOfBounds { len: usize, index: usize },

	/// An integer operation overflowed. Only used when the `checked-overflow` feature is enabled.
	IntegerOverflow,

	/// An illegal character appeared in the source code.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	NewTextError(NewTextError),

	/// A variable name was illegal.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	IllegalVariableName(IllegalVariableName),

	/// An error that doesn't fall into one of the other categories.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	Custom(Box<dyn std::error::Error + Send + Sync>),
}

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
	#[inline]
	fn from(err: io::Error) -> Self {
		Self::IoError(err)
	}
}

impl From<NewTextError> for Error {
	#[inline]
	fn from(err: NewTextError) -> Self {
		match err {
			#[cfg(feature = "compliance")]
			err => Self::NewTextError(err),
		}
	}
}

impl From<ParseError> for Error {
	#[inline]
	fn from(err: ParseError) -> Self {
		Self::ParseError(err)
	}
}

impl From<IllegalVariableName> for Error {
	#[inline]
	fn from(err: IllegalVariableName) -> Self {
		match err {
			#[cfg(feature = "compliance")]
			err => Self::IllegalVariableName(err),
		}
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		match self {
			Self::ParseError(err) => Some(err),
			Self::IoError(err) => Some(err),

			#[cfg(feature = "compliance")]
			Self::NewTextError(err) => Some(err),

			#[cfg(feature = "compliance")]
			Self::IllegalVariableName(err) => Some(err),

			#[cfg(feature = "extensions")]
			Self::Custom(err) => Some(err.as_ref()),

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
			Self::TypeError(kind, func) => write!(f, "invalid type {kind} given to {func}"),
			Self::DivisionByZero => write!(f, "division/modulo by zero"),
			Self::ParseError(err) => Display::fmt(&err, f),
			Self::Quit(status) => write!(f, "quitting with status code {status}"),
			Self::IntegerOverflow => write!(f, "integer under/overflow"),
			Self::IndexOutOfBounds { len, index } => {
				write!(f, "end index {index} is out of bounds for length {len}")
			}

			#[cfg(feature = "compliance")]
			Self::NewTextError(err) => Display::fmt(&err, f),

			#[cfg(feature = "compliance")]
			Self::IllegalVariableName(err) => Display::fmt(&err, f),

			#[cfg(feature = "extensions")]
			Self::Custom(err) => Display::fmt(&err, f),
		}
	}
}
