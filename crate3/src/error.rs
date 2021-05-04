use std::io;
use crate::{Number, Value, text::InvalidChar};

#[derive(Debug)]
pub enum Error {
	UndefinedVariable(String),
	UndefinedConversion { from: &'static str, to: &'static str },
	InvalidChar(InvalidChar),
	Io(io::Error),
	Quit { code: i32 },
	LengthOverflow { message: &'static str },
	InvalidOperand { func: char, operand: &'static str },
	Overflow { func: char, lhs: Number, rhs: Number },
	BadArgument { func: char, reason: &'static str },
	Custom(Box<dyn std::error::Error>)
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