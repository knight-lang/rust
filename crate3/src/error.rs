use std::io;
use crate::text::InvalidChar;

#[derive(Debug)]
pub enum Error {
	UndefinedVariable(String),
	UndefinedConversion { from: &'static str, to: &'static str },
	InvalidChar(InvalidChar),
	Io(io::Error)
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
	#[inline]
	fn from(err: io::Error) -> Self {
		Self::Io(err)
	}
}

impl From<InvalidChar> for Error {
	#[inline]
	fn from(err: InvalidChar) -> Self {
		Self::InvalidChar(err)
	}
}