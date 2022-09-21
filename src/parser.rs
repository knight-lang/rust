use crate::function::Function;
use crate::text::{Character, Text, TextSlice};
use crate::value::{Integer, List, Value};
use crate::variable::{IllegalVariableName, Variable};
use crate::{Ast, Environment};
use std::fmt::{self, Display, Formatter};

/// A type that represents errors that happen during parsing.
#[derive(Debug)]
#[must_use]
pub struct Error {
	/// What line the error occurred on.
	pub line: usize,

	/// What kind of error was it.
	pub kind: ErrorKind,
}

/// Type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type used to indicate an error whilst parsing Knight source code.
#[derive(Debug)]
#[non_exhaustive]
#[must_use]
pub enum ErrorKind {
	/// End of stream was reached before a token could be parsed.
	EmptySource,

	/// An unrecognized character was encountered.
	UnknownTokenStart(Character),

	/// A starting quote was found without an associated ending quote.
	UnterminatedString {
		/// The starting character of the quote (ie either `'` or `"`)
		quote: Character,
	},

	/// A function name was parsed, but an argument of its was missing.
	MissingArgument {
		/// The name of the function whose argument is missing.
		name: Text,

		/// The argument number.
		index: usize,
	},

	/// A left parenthesis didn't correspond to a matching a matching right one.
	UnmatchedLeftParen,

	/// A right parenthesis didn't correspond to a matching a matching left one.
	UnmatchedRightParen,

	/// A pair of parenthesis didn't enclose exactly one expression.
	DoesntEncloseExpression,

	/// A number literal was too large.
	IntegerLiteralOverflow,

	/// An unknown extension name was encountered.
	UnknownExtensionFunction(Text),

	/// A variable name wasn't valid for some reason
	///
	/// This is only returned when `verify-variable-names` is enabled.
	IllegalVariableName(IllegalVariableName),

	/// The source file wasn't exactly one expression.
	///
	/// This is only returned when `forbid-trailing-tokens` is enabled.
	TrailingTokens,
}

impl ErrorKind {
	/// Helper function to create a new [`Error`].
	pub fn error(self, line: usize) -> Error {
		Error { line, kind: self }
	}
}

impl Display for ErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::EmptySource => write!(f, "an empty source string was encountered"),
			Self::UnknownTokenStart(chr) => write!(f, "unknown token start {chr:?}"),
			Self::UnterminatedString { quote } => write!(f, "unterminated `{quote}` string"),
			Self::MissingArgument { name, index } => {
				write!(f, "missing argument {index} for function {name:?}")
			}
			Self::IntegerLiteralOverflow => write!(f, "integer literal overflowed max size"),
			Self::IllegalVariableName(ref err) => Display::fmt(&err, f),

			Self::UnmatchedLeftParen => write!(f, "an unmatched `(` was encountered"),
			Self::UnmatchedRightParen => write!(f, "an unmatched `)` was encountered"),
			Self::DoesntEncloseExpression => write!(f, "parens dont enclose an expression"),

			Self::UnknownExtensionFunction(ref name) => write!(f, "unknown extension {name}"),
			Self::TrailingTokens => write!(f, "trailing tokens encountered"),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "line {}: {}", self.line, self.kind)
	}
}

impl std::error::Error for Error {}

/// A type that handles parsing source code.
#[must_use]
pub struct Parser<'s, 'a, 'e> {
	source: &'s TextSlice,
	env: &'a mut Environment<'e>,
	line: usize,
}

impl<'s, 'a, 'e> Parser<'s, 'a, 'e> {
	/// Create a new `Parser` from the given source.
	pub fn new(source: &'s TextSlice, env: &'a mut Environment<'e>) -> Self {
		Self { source, line: 1, env }
	}

	fn error(&self, kind: ErrorKind) -> Error {
		kind.error(self.line)
	}

	fn peek(&self) -> Option<Character> {
		self.source.head()
	}

	fn advance(&mut self) -> Option<Character> {
		let mut chars = self.source.chars();

		let head = chars.next()?;
		if head == '\n' {
			self.line += 1;
		}

		self.source = chars.as_text();
		Some(head)
	}

	fn take_while<F: FnMut(Character) -> bool>(&mut self, mut func: F) -> &'s TextSlice {
		let start = self.source;

		while self.peek().map_or(false, &mut func) {
			self.advance();
		}

		start.get(..start.len() - self.source.len()).unwrap()
	}

	fn strip_whitespace_and_comments(&mut self) {
		loop {
			// strip all leading whitespace, if any.
			self.take_while(Character::is_whitespace);

			// If we're not at the start of a comment, break out
			if self.peek().map_or(true, |c| c != '#') {
				break;
			}

			// eat a comment.
			self.take_while(|chr| chr != '\n');
		}
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	pub fn parse_program(mut self) -> Result<Value<'e>> {
		let ret = self.parse()?;

		// If we forbid any trailing tokens, then see if we could have parsed anything else.
		#[cfg(feature = "forbid-trailing-tokens")]
		if !matches!(self.parse().map_err(|e| e.kind), Err(ErrorKind::EmptySource)) {
			return Err(self.error(ErrorKind::TrailingTokens));
		}

		Ok(ret)
	}

	fn parse_integer(&mut self) -> Result<Integer> {
		// The only way that `.parse` can fail is if we overflow, so we can safely map its error to
		// `IntegerLiteralOverflow`.
		self
			.take_while(Character::is_numeric)
			.parse()
			.map_err(|_| self.error(ErrorKind::IntegerLiteralOverflow))
	}

	fn parse_identifier(&mut self) -> Result<Variable<'e>> {
		let identifier = self.take_while(|chr| chr.is_lower() || chr.is_numeric());

		self.env.lookup(identifier).map_err(|err| self.error(ErrorKind::IllegalVariableName(err)))
	}

	fn parse_string(&mut self) -> Result<Text> {
		let quote = match self.advance() {
			Some(quote) if quote == '\'' || quote == '\"' => quote,
			_ => unreachable!(),
		};

		let start = self.line;
		let body = self.take_while(|chr| chr != quote);

		if self.advance() != Some(quote) {
			return Err(ErrorKind::UnterminatedString { quote }.error(start));
		}

		Ok(body.to_owned())
	}

	fn strip_function(&mut self) {
		if self.peek().expect("strip function at eof").is_upper() {
			// If it's a keyword function, then take all keyword characters.
			self.take_while(Character::is_upper);
		} else {
			// otherwise, only take that character.
			self.advance();
		}
	}

	fn parse_function(&mut self, func: &'e Function) -> Result<Ast<'e>> {
		// `MissingArgument` errors have their `line` field set to the beginning of the function
		// parsing.
		let start_line = self.line;

		let mut args = Vec::with_capacity(func.arity);

		for index in 0..func.arity {
			match self.parse() {
				Ok(arg) => args.push(arg),
				Err(Error { kind: ErrorKind::EmptySource, .. }) => {
					return Err(
						ErrorKind::MissingArgument { name: func.name.to_owned(), index }
							.error(start_line),
					)
				}
				Err(err) => return Err(err),
			}
		}

		Ok(Ast::new(func, args.into()))
	}

	fn parse_grouped_expression(&mut self) -> Result<Value<'e>> {
		use ErrorKind::*;

		let start = self.line;
		self.advance();

		match self.parse() {
			Ok(val) => {
				self.strip_whitespace_and_comments();
				if self.peek().map_or(false, |c| c == ')') {
					self.advance();
					return Ok(val);
				}
				Err(UnmatchedLeftParen.error(start))
			}
			Err(Error { kind: EmptySource, .. }) => Err(UnmatchedLeftParen.error(start)),
			Err(Error { kind: UnmatchedRightParen, .. }) => Err(DoesntEncloseExpression.error(start)),
			Err(other) => Err(other),
		}
	}

	fn parse(&mut self) -> Result<Value<'e>> {
		self.strip_whitespace_and_comments();

		let head = self.peek().ok_or_else(|| self.error(ErrorKind::EmptySource))?;

		match head.inner() {
			// Literals
			_ if head.is_numeric() => self.parse_integer().map(Value::from),
			_ if head.is_lower() => self.parse_identifier().map(Value::from),
			'\'' | '\"' => self.parse_string().map(Value::from),

			// Constants
			chr @ ('T' | 'F' | 'N' | '@') => {
				self.strip_function();
				Ok(match chr {
					'T' => Value::Boolean(true),
					'F' => Value::Boolean(false),
					'N' => Value::Null,
					'@' => Value::List(List::default()),
					_ => unreachable!(),
				})
			}

			// Parenthesis groupings
			'(' => self.parse_grouped_expression(),
			')' => Err(self.error(ErrorKind::UnmatchedRightParen)),

			// functions
			'X' => {
				let name = self.take_while(Character::is_upper);
				let &function = self
					.env
					.extensions()
					.get(name)
					.ok_or_else(|| self.error(ErrorKind::UnknownExtensionFunction(name.into())))?;

				self.parse_function(function).map(Value::from)
			}
			_ => {
				let &function = self
					.env
					.functions()
					.get(&head)
					.ok_or_else(|| self.error(ErrorKind::UnknownTokenStart(head)))?;

				self.strip_function();
				self.parse_function(function).map(Value::from)
			}
		}
	}
}
