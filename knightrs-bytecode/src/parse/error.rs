use super::Span;
use crate::strings::Encoding;
use std::fmt::{self, Display, Formatter};

/// Errors that can occur whilst parsing programs.
#[derive(Debug)]
pub struct Error<'src> {
	/// Where exactly the error happened.
	pub span: Span<'src>,

	/// What the problem was
	pub kind: ErrorKind,
}

/// Type alias for `Result<T, Error>`.
pub type Result<'src, T> = std::result::Result<T, Error<'src>>;

impl Display for Error<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}: {}", self.span.origin, self.kind)
	}
}

impl std::error::Error for Error<'_> {
	fn cause(&self) -> Option<&dyn std::error::Error> {
		self.kind.cause()
	}
}

/// Different kinds of errors that can occur when parsing
#[derive(Error, Debug)]
pub enum ErrorKind {
	/// The program source just had comments and whitespace
	#[error("there was nothing to parse.")]
	EmptySource,

	/// An unknown character appeared.
	#[error("character doesn't start a token: {0:?}")]
	UnknownTokenStart(char),

	/// An integer literal overflowed
	#[error("integer literal overflowed")]
	IntegerLiteralOverflow,

	/// A String was missing its ending quote.
	#[error("missing ending {0:?} quote")]
	MissingEndingQuote(char),

	/// A function was missing an argument.
	#[error("missing argument {1} for function {0:?}")]
	MissingArgument(char, usize),

	/// Assignment to non-variables.
	#[error("can only assign to variables")]
	CanOnlyAssignToVariables,

	/// Variable name was too long (only when in compliance mode).
	#[cfg(feature = "compliance")]
	#[error("variable name too long ({len} > {max}): {0:?}", len=.0.len(),
		max = crate::parser::VariableName::MAX_NAME_LEN)]
	VariableNameTooLong(String),

	/// Too many variables encountered (only when in compliance mode).
	#[cfg(feature = "compliance")]
	#[error("too many variables encountered (only {} allowed)", crate::vm::MAX_VARIABLE_COUNT)]
	TooManyVariables,

	/// Invalid source character (only when in compliance mode).
	#[cfg(feature = "compliance")]
	#[error("invalid character {1:?} for encoding {0:?}")]
	InvalidCharInEncoding(Encoding, char),

	/// Tokens after a single expression (only when in compliance mode).
	#[cfg(feature = "compliance")]
	#[error("there were additional tokens in the source")]
	TrailingTokens,

	/// A `(` was unrequited and didn't have a matching `)` (only when check-parens)
	#[cfg(feature = "check-parens")]
	#[error("missing matching `)` for paren")]
	MissingClosingParen,

	/// A `)` paren was encountered without a `(`. (only when check-parens)
	#[cfg(feature = "check-parens")]
	#[error("unmatched `)` found")]
	UnmatchedClosingParen,

	/// An unknown extension function was seen (only when extensiosn)
	#[cfg(feature = "extensions")]
	#[error("unknown extenision function: {0}")]
	UnknownExtensionFunction(String),

	// #[error("{0}")]
	// StringError(#[from] StringError),
	/// Custom errors
	#[cfg(feature = "extensions")]
	#[error("{0}")]
	Custom(Box<dyn std::error::Error>),
}
