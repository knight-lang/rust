use crate::parser::SourceLocation;
use crate::strings::{Encoding, StringError};
use std::fmt::{self, Display, Formatter};

/// An error that happens during program parsing.
///
/// This contains both the error itself (`kind`), and where it occurred (`whence`).
#[derive(Debug)]
pub struct ParseError<'path> {
	/// What kind of error occurred.
	pub kind: ParseErrorKind,

	/// Where the error happened.
	pub whence: SourceLocation<'path>,
}

impl std::error::Error for ParseError<'_> {
	fn cause(&self) -> Option<&dyn std::error::Error> {
		self.kind.cause()
	}
}

impl Display for ParseError<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}: {}", self.whence, self.kind)
	}
}

/// Different kinds of errors that can occur when parsing
#[derive(Error, Debug)]
pub enum ParseErrorKind {
	/// The program source just had comments and whitespace
	#[error("there was nothing to parse.")]
	EmptySource,

	/// An unknown character appeared.
	#[error("character doesn't start a token: {0:?}")]
	UnknownTokenStart(char),

	/// An integer literal overflowed
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

	#[cfg(feature = "compliance")]
	#[error("variable name too long ({len} > {max}): {0:?}", len=.0.len(),
		max = crate::parser::VariableName::MAX_NAME_LEN)]
	VariableNameTooLong(String),

	#[cfg(feature = "compliance")]
	#[error("too many variables encountered (only {} allowed)", crate::vm::MAX_VARIABLE_COUNT)]
	TooManyVariables,

	#[cfg(feature = "compliance")]
	#[error("invalid character {1:?} for encoding {0:?}")]
	InvalidCharInEncoding(Encoding, char),

	#[cfg(feature = "compliance")]
	#[error("there were additional tokens in the source")]
	TrailingTokens,

	#[cfg(feature = "check-parens")]
	#[error("missing matching `)` for paren")]
	MissingClosingParen,

	#[cfg(feature = "check-parens")]
	#[error("unmatched `)` found")]
	UnmatchedClosingParen,

	#[cfg(feature = "extensions")]
	#[error("unmatched `}}` found")]
	UnmatchedClosingBrace,

	#[cfg(feature = "extensions")]
	#[error("unknown extenision function: {0}")]
	UnknownExtensionFunction(String),

	#[cfg(feature = "extensions")]
	#[error("unknown \\\\ escape in X\" string: {0} ")]
	UnknownEscapeSequence(char),

	#[cfg(feature = "extensions")]
	#[error("Character {0} is not a hex character")]
	NotAHexChar(char),
}

impl ParseErrorKind {
	// this tuple is a huge hack. maybe when i remove it i can also remove `'filename`
	pub fn error<'path>(self, whence: SourceLocation<'path>) -> ParseError<'path> {
		ParseError { whence, kind: self }
	}
}
