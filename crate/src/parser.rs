use crate::knstr::{IllegalChar, KnStr};
use crate::{Environment, Value};
use std::fmt::{self, Display, Formatter};
use tap::prelude::*;

/// A type that represents errors that happen during parsing.
#[derive(Debug)]
pub struct ParseError {
	/// What line the error occurred on.
	pub line: usize,

	/// What kind of error was it.
	pub kind: ParseErrorKind,
}

/// The error type used to indicate an error whilst parsing Knight source code.
#[derive(Debug)]
pub enum ParseErrorKind {
	/// Indicates that the end of stream was reached before a value could be parsed.
	NothingToParse,

	/// Indicates that an invalid character was encountered.
	UnknownTokenStart(char),

	/// A starting quote was found without an associated ending quote.
	UnterminatedQuote {
		/// The starting character of the quote (ie either `'` or `"`)
		quote: char,
	},

	/// A function was parsed, but one of its arguments was not able to be parsed.
	MissingFunctionArgument {
		/// The function whose argument is missing.
		func: &'static crate::function::Function,

		/// The argument number.
		idx: usize,
	},

	/// A number literal was too large.
	IntegerLiteralOverflow,

	/// Indicates that there were some tokens trailing. Only used in `forbid-trailing-tokens` mode.
	#[cfg(feature = "forbid-trailing-tokens")]
	TrailingTokens,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		use ParseErrorKind::*;

		write!(f, "line {}: ", self.line)?;

		match self.kind {
			NothingToParse => write!(f, "a token was expected"),
			UnknownTokenStart(chr) => write!(f, "unknown token start {chr:?}"),
			UnterminatedQuote { quote } => write!(f, "unterminated `{quote}` quote encountered"),
			MissingFunctionArgument { func, idx } => {
				write!(f, "missing argument {idx} for function {:?}", func.name)
			}
			IntegerLiteralOverflow => write!(f, "integer literal overflowed max size"),
			#[cfg(feature = "forbid-trailing-tokens")]
			TrailingTokens => write!(f, "trailing tokens encountered"),
		}
	}
}

impl std::error::Error for ParseError {}

/// Parse source code.
#[derive(Debug, Clone)]
pub struct Parser<'a> {
	source: &'a KnStr,
	line: usize,
}

impl<'a> Parser<'a> {
	/// Create a new `Parser` from the given source.
	pub const fn new(source: &'a KnStr) -> Self {
		Self { source, line: 1 }
	}

	fn error(&self, kind: ParseErrorKind) -> ParseError {
		ParseError { line: self.line, kind }
	}

	fn peek(&self) -> Option<char> {
		self.source.chars().next()
	}

	fn advance(&mut self) -> Option<char> {
		let mut chars = self.source.chars();

		let ret = chars.next();
		if ret == Some('\n') {
			self.line += 1;
		}

		self.source = KnStr::new(chars.as_str()).unwrap();
		ret
	}

	fn take_while(&mut self, mut func: impl FnMut(char) -> bool) -> &'a KnStr {
		let start = self.source;

		while let Some(chr) = self.peek() {
			if func(chr) {
				self.advance();
			} else {
				break;
			}
		}

		KnStr::new(&start[..start.len() - self.source.len()]).unwrap()
	}

	fn strip(&mut self) {
		fn is_whitespace(chr: char) -> bool {
			" \r\n\t()[]{}:".contains(chr) || !cfg!(feature = "strict-charset") && chr.is_whitespace()
		}

		while !self.take_while(is_whitespace).is_empty() {
			if self.peek() == Some('#') {
				self.take_while(|chr| chr != '\n');
			}
		}
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	pub fn parse(mut self, env: &mut Environment) -> Result<Value, ParseError> {
		let return_value =
			self.parse_value(env)?.ok_or_else(|| self.error(ParseErrorKind::NothingToParse))?;

		#[cfg(feature = "forbid-trailing-tokens")]
		{
			self.strip();
			if !self.source.is_empty() {
				return Err(self.error(ParseErrorKind::TrailingTokens));
			}
		}

		Ok(return_value)
	}

	fn parse_value(&mut self, env: &mut Environment) -> Result<Option<Value>, ParseError> {
		fn is_lower(chr: char) -> bool {
			chr == '_'
				|| if cfg!(feature = "strict-charset") {
					chr.is_ascii_lowercase()
				} else {
					chr.is_lowercase()
				}
		}

		fn is_numeric(chr: char) -> bool {
			if cfg!(feature = "strict-charset") {
				chr.is_ascii_digit()
			} else {
				chr.is_numeric()
			}
		}
		fn is_upper(chr: char) -> bool {
			chr == '_'
				|| if cfg!(feature = "strict-charset") {
					chr.is_ascii_uppercase()
				} else {
					chr.is_uppercase()
				}
		}

		self.strip();

		let start = if let Some(chr) = self.peek() {
			chr
		} else {
			return Ok(None);
		};

		if is_upper(start) && start != '_' {
			self.take_while(is_upper);
		}

		match start {
			'0'..='9' => self
				.take_while(|chr| chr.is_ascii_digit())
				.parse::<crate::Integer>()
				.map(|num| Some(num.into()))
				.map_err(|_| self.error(ParseErrorKind::IntegerLiteralOverflow)),

			_ if is_lower(start) => {
				let ident = self.take_while(|chr| is_lower(chr) || is_numeric(chr));

				Ok(Some(env.lookup(ident).into()))
			}

			'\'' | '\"' => {
				let quote = start;
				self.advance();

				let body = self.take_while(|chr| chr != quote);

				if self.advance() != Some(quote) {
					return Err(self.error(ParseErrorKind::UnterminatedQuote { quote }));
				}

				Ok(Some(body.to_boxed().conv::<crate::SharedStr>().into()))
			}

			'T' | 'F' => Ok(Some((start == 'T').into())),
			'N' => Ok(Some(Value::Null)),
			#[cfg(feature = "arrays")]
			'Z' => Ok(Some(Value::Array(Default::default()))),

			_ => {
				if !is_upper(start) {
					self.advance();
				}

				let func = crate::function::fetch(start)
					.ok_or_else(|| self.error(ParseErrorKind::UnknownTokenStart(start)))?;

				let mut args = Vec::with_capacity(func.arity);

				for idx in 0..func.arity {
					if let Some(arg) = self.parse_value(env)? {
						args.push(arg);
					} else {
						return Err(self.error(ParseErrorKind::MissingFunctionArgument { func, idx }));
					}
				}

				Ok(Some(crate::Ast::new(func, args).into()))
			}
		}
	}
}
