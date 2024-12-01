use crate::parser::SourceLocation;
use crate::strings::{Encoding, StringError};
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct ParseError {
	pub whence: SourceLocation,
	pub kind: ParseErrorKind,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}: {}", self.whence, self.kind)
	}
}

impl std::error::Error for ParseError {}

#[derive(Error, Debug)]
pub enum ParseErrorKind {
	#[cfg(feature = "compliance")]
	#[error("variable name too long ({len} > {max}): {0:?}", len=.0.len(),
		max = crate::parser::VariableName::MAX_NAME_LEN)]
	VariableNameTooLong(crate::value::KString),

	#[cfg(feature = "compliance")]
	#[error("too many variables encountered (only {} allowed)", crate::vm::MAX_VARIABLE_COUNT)]
	TooManyVariables,

	#[cfg(feature = "compliance")]
	#[error("invalid character {1:?} for encoding {0:?}")]
	InvalidCharInEncoding(Encoding, char),

	#[cfg(feature = "compliance")]
	#[error("there were additional tokens in the source")]
	TrailingTokens,

	// There was nothing to parse
	#[error("there was nothing to parse.")]
	EmptySource,

	#[error("character doesn't start a token: {0:?}")]
	UnknownTokenStart(char),

	#[error("integer literal overflowed")]
	IntegerLiteralOverflow,

	#[error("missing ending {0:?} quote")]
	MissingEndingQuote(char),

	#[error("{0}")]
	StringError(#[from] StringError),

	#[error("missing argument {1} for function {0:?}")]
	MissingArgument(char, usize),

	#[error("can only assign to variables")]
	CanOnlyAssignToVariables,

	#[cfg(feature = "extensions")]
	#[error("unknown extenision function: {0}")]
	UnknownExtensionFunction(String),
}

impl ParseErrorKind {
	// this tuple is a huge hack. maybe when i remove it i can also remove `'filename`
	pub fn error(self, whence: SourceLocation) -> ParseError {
		ParseError { whence, kind: self }
	}
}
