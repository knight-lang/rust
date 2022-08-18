use crate::text::IllegalByte;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum Error {
	NoConversion { from: &'static str, to: &'static str },
	IllegalByte(IllegalByte),
	UndefinedVariable(String),
	IoError(io::Error),
	DomainError(&'static str),
	TypeError(char, &'static str),
	IntegerOverflow,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		Self::IoError(err)
	}
}

impl From<IllegalByte> for Error {
	fn from(err: IllegalByte) -> Self {
		Self::IllegalByte(err)
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		match self {
			Self::IllegalByte(err) => Some(err),
			Self::IoError(err) => Some(err),
			_ => None,
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::NoConversion { from, to } => write!(f, "undefined conversion from {from} to {to}"),
			Self::IllegalByte(err) => Display::fmt(&err, f),
			Self::UndefinedVariable(name) => write!(f, "undefined variable {name} was accessed"),
			Self::IoError(err) => write!(f, "an io error occurred: {err}"),
			Self::DomainError(err) => write!(f, "an domain error occurred: {err}"),
			Self::TypeError(name, kind) => write!(f, "invalid kind {kind} for function {name:?}"),
			Self::IntegerOverflow => write!(f, "integer under/overflow"),
		}
	}
}
