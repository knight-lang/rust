use std::io;
use std::fmt::{self, Display, Formatter};
use crate::{ParseError, text::InvalidChar};
use std::error::Error as ErrorTrait;

/// An error occurred whilst executing a knight program.
#[derive(Debug)]
pub enum Error {
	/// A division (or modulus) by zero was attempted.
	DivisionByZero {
		/// What kind of error it is---power, modulo, or division.
		kind: &'static str
	},

	/// An unknown identifier was attempted to be dereferenced.
	UnknownIdentifier {
		/// The identifier at fault.
		identifier: Box<str>
	},

	/// A function was executed with an invalid operand.
	InvalidOperand {
		/// The function that was attempted.
		func: char,

		/// The type of the operand.
		operand: &'static str
	},

	/// A conversion was attempted for a type which doesn't implement it.
	///
	/// This is only used for [`Value::Variable`](crate::Value::Variable) and [`Value::Function`](
	/// crate::Value::Function), as all other types have well-defined conversion semantics.
	UndefinedConversion {
		/// The resulting type, had the conversion been defined.
		into: &'static str,

		/// The kind that didnt implement the conversion.
		kind: &'static str
	},

	/// A checked operation failed.
	#[cfg(feature = "checked-overflow")]
	Overflow {
		/// Which function overflowed.
		func: char,

		/// The left-hand-side of the function.
		lhs: crate::Number,

		/// The right-hand-side of the function.
		rhs: crate::Number,
	},

	/// Exit with the given status code.
	Quit(i32),

	/// An error occurred whilst parsing (i.e. `EVAL` failed).
	Parse(ParseError),

	/// An invalid string was encountered.
	InvalidString(InvalidChar),

	/// An i/o error occurred (i.e. `` ` `` or `PROMPT` failed).
	Io(io::Error),

	/// An error class that can be used to raise other, custom errors.
	Custom(Box<dyn ErrorTrait>),
}

impl From<ParseError> for Error {
	#[inline]
	fn from(err: ParseError) -> Self {
		Self::Parse(err)
	}
}

impl From<io::Error> for Error {
	#[inline]
	fn from(err: io::Error) -> Self {
		Self::Io(err)
	}
}

impl From<InvalidChar> for Error {
	#[inline]
	fn from(err: InvalidChar) -> Self {
		Self::InvalidString(err)
	}
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::NothingToParse => write!(f, "a token was expected."),
			Self::UnknownTokenStart { chr, line } => write!(f, "line {}: unknown token start {:?}.", line, chr),
			Self::UnterminatedQuote { line } => write!(f, "line {}: unterminated quote encountered.", line),
			Self::MissingFunctionArgument { func, number, line }
				=> write!(f, "line {}: missing argument {} for function {:?}.", line, number, func),
			Self::InvalidString { line, err } => write!(f, "line {}: {}", line, err)
		}
	}
}

impl ErrorTrait for ParseError {
	fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
		match self {
			Self::InvalidString { err, .. } => Some(err),
			_ => None
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::DivisionByZero { kind } => write!(f, "invalid {} with zero.", kind),
			Self::UnknownIdentifier { identifier } => write!(f, "identifier {:?} is undefined.", identifier),
			Self::InvalidOperand { func, operand } => write!(f, "invalid operand kind {:?} for function {:?}.", operand, func),
			Self::UndefinedConversion { kind, into } => write!(f, "invalid conversion into {:?} for kind {:?}.", kind, into),
			#[cfg(feature = "checked-overflow")]
			Self::Overflow { func, lhs, rhs } => write!(f, "Expression '{} {} {}' overflowed", lhs, func, rhs),
			Self::Quit(code) => write!(f, "exit with status {}", code),
			Self::Parse(err) => Display::fmt(err, f),
			Self::InvalidString(err) => Display::fmt(err, f),
			Self::Io(err) => write!(f, "i/o error: {}", err),
			Self::Custom(err) => Display::fmt(err, f),
		}
	}
}

impl ErrorTrait for Error {
	fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
		match self {
			Self::Parse(err) => Some(err),
			Self::Io(err) => Some(err),
			Self::InvalidString(err) => Some(err),
			Self::Custom(err) => Some(err.as_ref()),
			_ => None
		}
	}
}
