use crate::knstr::IllegalChar;
use crate::parser::ParseError;
use crate::KnStr;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum Error {
	NoConversion {
		from: &'static str,
		to: &'static str,
	},
	IllegalChar(IllegalChar),
	UndefinedVariable(Box<KnStr>),
	IoError(io::Error),
	DomainError(&'static str),
	TypeError(&'static str),
	DivisionByZero,
	#[cfg(feature = "checked-overflow")]
	IntegerOverflow,
	ParseError(ParseError),

	Quit(i32),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		Self::IoError(err)
	}
}

impl From<IllegalChar> for Error {
	fn from(err: IllegalChar) -> Self {
		Self::IllegalChar(err)
	}
}

impl From<ParseError> for Error {
	fn from(err: ParseError) -> Self {
		Self::ParseError(err)
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		match self {
			Self::IllegalChar(err) => Some(err),
			Self::ParseError(err) => Some(err),
			Self::IoError(err) => Some(err),
			_ => None,
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::NoConversion { from, to } => write!(f, "undefined conversion from {from} to {to}"),
			Self::IllegalChar(err) => Display::fmt(&err, f),
			Self::UndefinedVariable(name) => write!(f, "undefined variable {name} was accessed"),
			Self::IoError(err) => write!(f, "an io error occurred: {err}"),
			Self::DomainError(err) => write!(f, "an domain error occurred: {err}"),
			Self::TypeError(kind) => write!(f, "invalid type {kind} given"),
			Self::DivisionByZero => write!(f, "division/modulo by zero"),
			Self::ParseError(err) => Display::fmt(&err, f),
			Self::Quit(status) => write!(f, "quitting with status code {status}"),

			#[cfg(feature = "checked-overflow")]
			Self::IntegerOverflow => write!(f, "integer under/overflow"),
		}
	}
}
