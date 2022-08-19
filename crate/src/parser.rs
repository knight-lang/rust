use crate::knightstr::{IllegalChar, KnightStr};
use crate::{Environment, Value};
use std::fmt::{self, Display, Formatter};
use tap::prelude::*;

#[derive(Debug)]
pub struct ParseError {
	pub line: usize,
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

	/// A number literal was too large
	IntegerLiteralOverflow,

	/// Indicates that there were some tokens trailing
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

#[derive(Debug, Clone)]
pub struct Parser<'a> {
	source: &'a KnightStr,
	line: usize,
}

fn is_whitespace(chr: char) -> bool {
	" \r\n\t()[]{}:".contains(chr) || !cfg!(feature = "strict-charset") && chr.is_whitespace()
}

impl<'a> Parser<'a> {
	pub const fn new(source: &'a KnightStr) -> Self {
		Self { source, line: 1 }
	}

	pub const fn source(&self) -> &KnightStr {
		&self.source
	}

	pub const fn line(&self) -> usize {
		self.line
	}

	fn error(&self, kind: ParseErrorKind) -> ParseError {
		ParseError { line: self.line(), kind }
	}

	pub fn peek(&self) -> Option<char> {
		self.source.chars().next()
	}

	pub fn advance(&mut self) -> Option<char> {
		let mut chars = self.source.chars();

		let ret = chars.next();
		if ret == Some('\n') {
			self.line += 1;
		}

		self.source = KnightStr::new(chars.as_str()).unwrap();
		ret
	}

	pub fn take_while(&mut self, mut func: impl FnMut(char) -> bool) -> &'a KnightStr {
		let start = self.source;

		while let Some(chr) = self.peek() {
			if func(chr) {
				self.advance();
			} else {
				break;
			}
		}

		KnightStr::new(&start[..start.len() - self.source.len()]).unwrap()
	}

	pub fn strip(&mut self) {
		while !self.take_while(is_whitespace).is_empty() {
			if self.peek() == Some('#') {
				self.take_while(|chr| chr != '\n');
			}
		}
	}

	pub fn parse_program(mut self, env: &mut Environment<'_>) -> Result<Value, ParseError> {
		let return_value =
			self.parse(env)?.ok_or_else(|| self.error(ParseErrorKind::NothingToParse))?;

		#[cfg(feature = "forbid-trailing-tokens")]
		{
			self.strip();
			if !self.source.is_empty() {
				return Err(self.error(ParseErrorKind::TrailingTokens));
			}
		}

		Ok(return_value)
	}

	pub fn parse(&mut self, env: &mut Environment<'_>) -> Result<Option<Value>, ParseError> {
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

		match start {
			'0'..='9' => self
				.take_while(|chr| chr.is_ascii_digit())
				.parse::<crate::Number>()
				.map(|num| Some(num.into()))
				.map_err(|_| self.error(ParseErrorKind::IntegerLiteralOverflow)),

			_ if is_lower(start) => self
				.take_while(|chr| is_lower(chr) || is_numeric(chr))
				.pipe(|name| Ok(Some(env.lookup(name).into()))),

			'\'' | '\"' => {
				let quote = start;
				self.advance();

				let body = self.take_while(|chr| chr != quote);

				if self.advance() != Some(quote) {
					return Err(self.error(ParseErrorKind::UnterminatedQuote { quote }));
				}

				Ok(Some(body.to_boxed().conv::<crate::Text>().into()))
			}

			'T' | 'F' => {
				self.take_while(is_upper);
				Ok(Some((start == 'T').into()))
			}
			'N' => {
				self.take_while(is_upper);
				Ok(Some(Value::Null))
			}

			_ => {
				if is_upper(start) {
					self.take_while(is_upper);
				} else {
					self.advance();
				}

				let func = crate::function::fetch(start)
					.ok_or_else(|| self.error(ParseErrorKind::UnknownTokenStart(start)))?;

				let mut args = Vec::with_capacity(func.arity);

				for idx in 0..func.arity {
					if let Some(arg) = self.parse(env)? {
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
