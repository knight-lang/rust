use crate::text::{IllegalChar, SharedText, Text};
use crate::variable::IllegalVariableName;
use crate::{Environment, Integer, Value};
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
	EmptySource,

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

	/// A variable name wasn't valid for some reason
	IllegalVariableName(IllegalVariableName),

	/// Indicates that there were some tokens trailing.
	#[cfg(feature = "forbid-trailing-tokens")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "forbid-trailing-tokens")))]
	TrailingTokens,

	/// An unknown extension was encountered.
	#[cfg(feature = "extension-functions")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "extension-functions")))]
	UnknownExtensionFunction(SharedText),
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		use ParseErrorKind::*;

		write!(f, "line {}: ", self.line)?;

		match self.kind {
			EmptySource => write!(f, "an empty source string was encountered"),
			UnknownTokenStart(chr) => write!(f, "unknown token start {chr:?}"),
			UnterminatedQuote { quote } => write!(f, "unterminated `{quote}` quote encountered"),
			MissingFunctionArgument { func, idx } => {
				write!(f, "missing argument {idx} for function {:?}", func.name)
			}
			IntegerLiteralOverflow => write!(f, "integer literal overflowed max size"),
			IllegalVariableName(ref err) => Display::fmt(&err, f),

			#[cfg(feature = "forbid-trailing-tokens")]
			TrailingTokens => write!(f, "trailing tokens encountered"),

			#[cfg(feature = "extension-functions")]
			UnknownExtensionFunction(ref name) => write!(f, "unknown extension {name}"),
		}
	}
}

impl std::error::Error for ParseError {}

/// Parse source code.
#[derive(Debug, Clone)]
pub struct Parser<'a> {
	source: &'a Text,
	line: usize,
}

fn is_whitespace(chr: char) -> bool {
	" \r\n\t()[]{}:".contains(chr) || !cfg!(feature = "strict-charset") && chr.is_whitespace()
}

pub(crate) fn is_lower(chr: char) -> bool {
	chr == '_'
		|| if cfg!(feature = "strict-charset") {
			chr.is_ascii_lowercase()
		} else {
			chr.is_lowercase()
		}
}

pub(crate) fn is_numeric(chr: char) -> bool {
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

impl<'a> Parser<'a> {
	/// Create a new `Parser` from the given source.
	pub const fn new(source: &'a Text) -> Self {
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

		let head = chars.next()?;
		if head == '\n' {
			self.line += 1;
		}

		self.source = chars.as_text();
		Some(head)
	}

	fn take_while(&mut self, mut func: impl FnMut(char) -> bool) -> &'a Text {
		let start = self.source;

		while self.peek().map_or(false, &mut func) {
			self.advance();
		}

		start.get(..start.len() - self.source.len()).unwrap()
	}

	fn strip(&mut self) {
		loop {
			self.take_while(is_whitespace).is_empty();

			if self.peek() != Some('#') {
				break;
			}

			self.take_while(|chr| chr != '\n');
		}
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	pub fn parse(mut self, env: &mut Environment) -> Result<Value, ParseError> {
		let ret = self.parse_value(env)?;

		#[cfg(feature = "forbid-trailing-tokens")]
		{
			self.strip();
			if !self.source.is_empty() {
				return Err(self.error(ParseErrorKind::TrailingTokens));
			}
		}

		Ok(ret)
	}

	fn parse_function(
		&mut self,
		func: &'static crate::Function,
		env: &mut Environment,
	) -> Result<Value, ParseError> {
		if is_upper(func.name) {
			self.take_while(is_upper);
		} else {
			self.advance();
		}

		let mut args = Vec::with_capacity(func.arity);
		let start_line = self.line;

		for idx in 0..func.arity {
			match self.parse_value(env) {
				Ok(arg) => args.push(arg),
				Err(ParseError { kind: ParseErrorKind::EmptySource, .. }) => {
					return Err(ParseError {
						line: start_line,
						kind: ParseErrorKind::MissingFunctionArgument { func, idx },
					})
				}
				Err(err) => return Err(err),
			}
		}

		Ok(crate::Ast::new(func, args).into())
	}

	fn parse_value(&mut self, env: &mut Environment) -> Result<Value, ParseError> {
		self.strip();

		match self.peek().ok_or_else(|| self.error(ParseErrorKind::EmptySource))? {
			// integer literals
			'0'..='9' => self
				.take_while(is_numeric)
				.parse::<Integer>()
				.map(Value::from)
				.map_err(|_| self.error(ParseErrorKind::IntegerLiteralOverflow)),

			// identifiers
			start if is_lower(start) => {
				let identifier = self.take_while(|chr| is_lower(chr) || is_numeric(chr));

				env.lookup(identifier)
					.map(Value::from)
					.map_err(|err| self.error(ParseErrorKind::IllegalVariableName(err)))
			}

			// strings
			quote @ ('\'' | '\"') => {
				self.advance();

				let body = self.take_while(|chr| chr != quote);

				if self.advance() != Some(quote) {
					return Err(self.error(ParseErrorKind::UnterminatedQuote { quote }));
				}

				Ok(body.conv::<crate::SharedText>().into())
			}

			// booleans
			start @ ('T' | 'F') => {
				self.take_while(is_upper);
				Ok((start == 'T').into())
			}
			// null
			'N' => {
				self.take_while(is_upper);
				Ok(Value::default())
			}
			#[cfg(feature = "arrays")]
			'Z' => {
				self.take_while(is_upper);
				Ok(Value::Array(Default::default()))
			}

			#[cfg(feature = "extension-functions")]
			'X' => {
				self.advance();
				let name = self.take_while(is_upper);
				let ext = env
					.extensions()
					.get(name)
					.ok_or_else(|| self.error(ParseErrorKind::UnknownExtensionFunction(name.into())))?;

				self.parse_function(ext, env)
			}

			// functions
			name => {
				let func = crate::function::fetch(name)
					.ok_or_else(|| self.error(ParseErrorKind::UnknownTokenStart(name)))?;

				self.parse_function(func, env)
			}
		}
	}
}
