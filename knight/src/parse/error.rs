use std::fmt::{self, Display, Formatter};
use std::error::Error;

#[derive(Debug)]
pub struct ParseError {
	pub line: usize,
	pub kind: ParseErrorKind
}

#[derive(Debug)]
pub enum ParseErrorKind {
	Custom(Box<dyn Error>),
	MissingTerminatingQuote,
	NothingToBeParsed,
	UnknownTokenStart
}

pub type ParseResult<T> = Result<T, ParseError>;

impl Error for ParseError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self.kind {
			ParseErrorKind::Custom(ref err) => Some(err.as_ref()),
			_ => None
		}
	}
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "line {}: {}", self.line, self.kind)
	}
}

impl Display for ParseErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Custom(err) => write!(f, "{}", err),
			_ => todo!()
		}
	}
}