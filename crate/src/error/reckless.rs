use std::hint::unreachable_unchecked;
use std::io;
use std::fmt::{self, Display, Formatter};
use crate::{ParseError, text::InvalidChar};
use std::error::Error as ErrorTrait;

#[derive(Debug)]
pub enum Error { }

impl From<ParseError> for Error {
	#[inline]
	fn from(_: ParseError) -> Self {
		unsafe {
			unreachable_unchecked()
		}
	}
}

impl From<io::Error> for Error {
	#[inline]
	fn from(_: io::Error) -> Self {
		unsafe {
			unreachable_unchecked()
		}
	}
}

impl From<InvalidChar> for Error {
	#[inline]
	fn from(_: InvalidChar) -> Self {
		unsafe {
			unreachable_unchecked()
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		unsafe {
			unreachable_unchecked()
		}
	}
}

impl ErrorTrait for Error {
	fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
		unsafe {
			unreachable_unchecked()
		}
	}
}