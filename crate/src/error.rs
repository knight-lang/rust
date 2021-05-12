use std::io;
use std::fmt::{self, Display, Formatter};
use crate::{number::MathError, stream::ParseError, text::InvalidChar};
use std::error::Error as ErrorTrait;

/// An error occurred whilst executing a knight program.
#[derive(Debug)]
pub enum Error {
	/// A problem occured with numbers (eg division by zero, overflow, etc.)
	Math(MathError),

	/// a valid type was given, but it wasnt in the correct domain.
	Domain(&'static str),

	#[cfg(feature="checked-overflow")]
	Overflow(&'static str),

	/// An unknown identifier was attempted to be dereferenced.
	UnknownIdentifier {
		/// The identifier at fault.
		identifier: Box<str>
	},

	/// A function was executed with an invalid operand.
	Type {
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
	BinaryOverflow {
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

	#[cfg(feature="checked-overflow")]
	TextConversionOverflow,

	/// An error class that can be used to raise other, custom errors.
	Custom(Box<dyn ErrorTrait>),
}

/// A type alias for [`std::result::Result`].
pub type Result<T> = std::result::Result<T, Error>;

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

impl From<MathError> for Error {
	#[inline]
	fn from(err: MathError) -> Self {
		Self::Math(err)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Math(err) => Display::fmt(err, f),
			Self::Domain(cause) => write!(f, "{}", cause),
			Self::UnknownIdentifier { identifier } => write!(f, "identifier {:?} is undefined.", identifier),
			Self::Type { func, operand } => write!(f, "invalid operand kind {:?} for function {:?}.", operand, func),
			Self::UndefinedConversion { kind, into } => write!(f, "invalid conversion into {:?} for kind {:?}.", kind, into),
			Self::Quit(code) => write!(f, "exit with status {}", code),
			Self::Parse(err) => Display::fmt(err, f),
			Self::InvalidString(err) => Display::fmt(err, f),
			Self::Io(err) => write!(f, "i/o error: {}", err),
			#[cfg(feature="checked-overflow")]
			Self::TextConversionOverflow => write!(f, "text to number conversion overflowed."),
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
			Self::Math(err) => Some(err),
			Self::Custom(err) => Some(err.as_ref()),
			_ => None
		}
	}
}
