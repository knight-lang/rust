use crate::text::{Text, TextSlice};
use crate::variable::IllegalVariableName;
use crate::{Ast, Environment, Integer, Value};
use std::fmt::{self, Display, Formatter};

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
#[non_exhaustive]
pub enum ParseErrorKind {
	/// Indicates that the end of stream was reached before a value could be parsed.
	EmptySource,

	/// Indicates that an invalid character was encountered.
	InvalidCharacter(char),

	/// A starting quote was found without an associated ending quote.
	UnterminatedQuote {
		/// The starting character of the quote (ie either `'` or `"`)
		quote: char,
	},

	/// A function was parsed, but one of its arguments was not able to be parsed.
	MissingArgument {
		/// The name of the function whose argument is missing.
		name: &'static str,

		/// The argument number.
		index: usize,
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
	UnknownExtensionFunction(Text),
}

impl Display for ParseErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::EmptySource => write!(f, "an empty source string was encountered"),
			Self::InvalidCharacter(chr) => write!(f, "unknown token start {chr:?}"),
			Self::UnterminatedQuote { quote } => write!(f, "unterminated `{quote}` quote encountered"),
			Self::MissingArgument { name, index } => {
				write!(f, "missing argument {index} for function {name:?}")
			}
			Self::IntegerLiteralOverflow => write!(f, "integer literal overflowed max size"),
			Self::IllegalVariableName(ref err) => Display::fmt(&err, f),

			#[cfg(feature = "forbid-trailing-tokens")]
			Self::TrailingTokens => write!(f, "trailing tokens encountered"),

			#[cfg(feature = "extension-functions")]
			Self::UnknownExtensionFunction(ref name) => write!(f, "unknown extension {name}"),
		}
	}
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "line {}: {}", self.line, self.kind)
	}
}

impl std::error::Error for ParseError {}

/// Parse source code.
#[derive(Debug, Clone)]
pub struct Parser<'a> {
	source: &'a TextSlice,
	line: usize,
}

fn is_whitespace(chr: char) -> bool {
	" \r\n\t():".contains(chr) || !cfg!(feature = "strict-charset") && chr.is_whitespace()
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
	pub const fn new(source: &'a TextSlice) -> Self {
		Self { source, line: 1 }
	}

	fn error(&self, kind: ParseErrorKind) -> ParseError {
		ParseError { line: self.line, kind }
	}

	fn peek(&self) -> Option<char> {
		self.source.into_iter().next()
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

	fn take_while(&mut self, mut func: impl FnMut(char) -> bool) -> &'a TextSlice {
		let start = self.source;

		while self.peek().map_or(false, &mut func) {
			self.advance();
		}

		start.get(..start.len() - self.source.len()).unwrap()
	}

	fn strip(&mut self) {
		loop {
			// strip all leading whitespace, if any.
			self.take_while(is_whitespace);

			// If we don't have a comment afterwards, nothing left to strip
			if self.peek() != Some('#') {
				break;
			}

			self.take_while(|chr| chr != '\n');
		}
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	pub fn parse<'e>(mut self, env: &mut Environment<'e>) -> Result<Value<'e>, ParseError> {
		let ret = self.parse_value(env)?;

		// If we forbid any trailing tokens, then see if we could have parsed anything else.
		#[cfg(feature = "forbid-trailing-tokens")]
		if !matches!(self.parse_value(env), Err(Error::EmptySource)) {
			return Err(self.error(ParseErrorKind::TrailingTokens));
		}

		Ok(ret)
	}

	fn parse_integer(&mut self) -> Result<Integer, ParseError> {
		// The only way that `.parse` can fail is if we overflow, so we can safely map its error to
		// `IntegerLiteralOverflow`.
		self
			.take_while(is_numeric)
			.parse()
			.map_err(|_| self.error(ParseErrorKind::IntegerLiteralOverflow))
	}

	fn parse_identifier<'e>(
		&mut self,
		env: &mut Environment<'e>,
	) -> Result<crate::Variable<'e>, ParseError> {
		let identifier = self.take_while(|chr| is_lower(chr) || is_numeric(chr));

		env.lookup(identifier).map_err(|err| self.error(ParseErrorKind::IllegalVariableName(err)))
	}

	fn parse_string(&mut self) -> Result<Text, ParseError> {
		let quote = match self.advance() {
			Some(quote @ ('\'' | '\"')) => quote,
			_ => unreachable!(),
		};

		let start = self.line;
		let body = self.take_while(|chr| chr != quote);

		if self.advance() == Some(quote) {
			Ok(body.into())
		} else {
			Err(ParseError { line: start, kind: ParseErrorKind::UnterminatedQuote { quote } })
		}
	}

	fn parse_function<'e>(
		&mut self,
		func: &'e crate::Function,
		env: &mut Environment<'e>,
	) -> Result<Ast<'e>, ParseError> {
		// If it's a keyword function, then take all keyword characters.
		if is_upper(func.name.chars().next().expect("function has empty name?")) {
			self.take_while(is_upper);
		} else {
			self.advance();
		}

		// `MissingArgument` errors have their `line` field set to the beginning of the function
		// parsing.
		let start_line = self.line;

		// Parse out all the arguments for the function, converting `EmptySource` errors into
		// `MissingArgument` errors.
		let args = (0..func.arity)
			.map(|index| match self.parse_value(env) {
				Ok(arg) => Ok(arg),
				Err(ParseError { kind: ParseErrorKind::EmptySource, .. }) => Err(ParseError {
					line: start_line,
					kind: ParseErrorKind::MissingArgument { name: func.name, index },
				}),
				Err(err) => Err(err),
			})
			.collect::<Result<_, _>>()?;

		// Looks like we're good, make a new AST.
		Ok(Ast::new(func, args))
	}

	fn parse_value<'e>(&mut self, env: &mut Environment<'e>) -> Result<Value<'e>, ParseError> {
		self.strip();

		match self.peek().ok_or_else(|| self.error(ParseErrorKind::EmptySource))? {
			'0'..='9' => self.parse_integer().map(Value::from),
			start if is_lower(start) => self.parse_identifier(env).map(Value::from),
			'\'' | '\"' => self.parse_string().map(Value::from),

			// constants
			start @ ('T' | 'F') => {
				self.take_while(is_upper);
				Ok((start == 'T').into())
			}
			'N' => {
				self.take_while(is_upper);
				Ok(Value::default())
			}
			'@' => {
				self.advance();
				Ok(Value::List(Default::default()))
			}

			#[cfg(feature = "extension-functions")]
			'X' => {
				self.advance();
				let name = self.take_while(is_upper);
				let ext = env
					.extensions()
					.get(name)
					.ok_or_else(|| self.error(ParseErrorKind::UnknownExtensionFunction(name.into())))?;

				self.parse_function(ext, env).map(Value::from)
			}

			// functions
			name => {
				let func = env
					.lookup_function(name)
					.ok_or_else(|| self.error(ParseErrorKind::InvalidCharacter(name)))?;

				self.parse_function(func, env).map(Value::from)
			}
		}
	}
}
