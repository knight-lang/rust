use crate::{Environment, Value};
use std::fmt::{self, Display, Formatter};

mod functions;
mod stream;
pub use functions::Functions;
pub use stream::Stream;

#[derive(Debug)]
pub struct ParseError {
	pub line: usize,
	pub kind: ParseErrorKind
}

#[derive(Debug)]
pub enum ParseErrorKind {
	Custom(Box<dyn std::error::Error>),
	MissingTerminatingQuote,
	NothingToBeParsed,
	UnknownTokenStart
}

pub type ParseResult<T> = Result<T, ParseError>;

impl std::error::Error for ParseError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
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

pub struct Parser<'env, 'stream> {
	env: &'env Environment,
	stream: Stream<'stream>
}

impl<'env> Parser<'env, '_> {
	pub fn parse(&mut self) -> ParseResult<Value<'env>> {
		let prefix = self.stream.next().ok_or_else(|| self.stream.error(ParseErrorKind::NothingToBeParsed))?;

		let func = self.env.functions().get(prefix).ok_or_else(|| self.stream.error(ParseErrorKind::UnknownTokenStart))?;

		(func)(prefix, self)
	}
}
