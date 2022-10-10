use crate::env::{IllegalVariableName, Variable};
use crate::text::{Character, Text, TextSlice};
use crate::value::{Integer, List, Value};
use crate::{Ast, Environment};
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

/// A type that handles parsing source code.
#[must_use]
pub struct Parser<'s, 'e> {
	source: &'s TextSlice,
	env: &'s mut Environment<'e>,
	line: usize,
}

/// A trait that indicates that something can be parsed.
pub trait Parsable<'s, 'e>: Sized {
	fn parse(parser: &mut Parser<'s, 'e>) -> Result<Option<Self>>;

	// fn parse_fn() -> Rc<ParseFn<'e>>
	// where
	// 	Value<'e>: From<Self>,
	// 	'e: 's,
	// {
	// 	Rc::new(|parser: &mut Parser<'_, 'e>| Ok(Self::parse(parser)?.map(Value::from))) as _
	// }
}

pub type ParseFn<'e> = dyn Fn(&mut Parser<'_, 'e>) -> Result<Option<Value<'e>>>;

pub(crate) fn default<'e>(flags: &crate::env::Flags) -> Vec<Rc<ParseFn<'e>>> {
	let _ = flags;

	let mut parsers = Vec::<Rc<ParseFn>>::new();

	parsers.push(Rc::new(|parser: &mut Parser<'_, 'e>| {
		parser.strip_whitespace_and_comments();
		Ok(None)
	}));

	parsers.push(Rc::new(|parser: &mut Parser<'_, 'e>| {
		if parser.advance_if('(').is_some() {
			parser.parse_grouped_expression().map(Some)
		} else if parser.advance_if(')').is_some() {
			Err(parser.error(ErrorKind::UnmatchedRightParen))
		} else {
			Ok(None)
		}
	}));

	macro_rules! parsers {
		($($ty:ty),*) => {
			$(parsers.push(Rc::new(|parser: &mut Parser<'_, 'e>| Ok(<$ty>::parse(parser)?.map(Value::from))) as _);)*
		};
	}

	parsers!(Integer, Text, Variable, crate::value::Boolean, crate::value::Null, List, Ast);

	parsers
}

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
	UnterminatedText {
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
	/// This is only returned when the `verify-variable-names` is enabled.
	IllegalVariableName(IllegalVariableName),

	/// The source file wasn't exactly one expression.
	///
	/// This is only returned when `forbid-trailing-tokens` is enabled.
	TrailingTokens,

	Custom(Box<dyn std::error::Error>), // TODO: make this be the `cause`
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
			Self::UnterminatedText { quote } => write!(f, "unterminated `{quote}` text"),
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
			Self::Custom(err) => Display::fmt(err, f),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "line {}: {}", self.line, self.kind)
	}
}

impl std::error::Error for Error {}

pub trait AdvanceIfCondition {
	fn should_advance(self, chr: Character) -> bool;
}
impl<T: FnOnce(Character) -> bool> AdvanceIfCondition for T {
	fn should_advance(self, chr: Character) -> bool {
		self(chr)
	}
}

impl AdvanceIfCondition for Character {
	fn should_advance(self, chr: Character) -> bool {
		chr == self
	}
}

impl AdvanceIfCondition for char {
	fn should_advance(self, chr: Character) -> bool {
		chr == self
	}
}

impl<'s, 'e> Parser<'s, 'e> {
	/// Create a new `Parser` from the given source.
	pub fn new(source: &'s TextSlice, env: &'s mut Environment<'e>) -> Self {
		Self { source, line: 1, env }
	}

	pub fn line(&self) -> usize {
		self.line
	}

	pub fn env(&mut self) -> &mut Environment<'e> {
		self.env
	}

	pub fn error(&self, kind: ErrorKind) -> Error {
		kind.error(self.line)
	}

	pub fn peek(&self) -> Option<Character> {
		self.source.head()
	}

	pub fn advance_if<F: AdvanceIfCondition>(&mut self, cond: F) -> Option<Character> {
		let mut chars = self.source.chars();

		let head = chars.next()?;
		if !cond.should_advance(head) {
			return None;
		}

		if head == '\n' {
			self.line += 1;
		}

		self.source = chars.as_text();
		Some(head)
	}

	pub fn advance(&mut self) -> Option<Character> {
		self.advance_if(|_| true)
	}

	pub fn take_while<F: FnMut(Character) -> bool>(&mut self, mut func: F) -> Option<&'s TextSlice> {
		let start = self.source;

		while self.peek().map_or(false, &mut func) {
			self.advance();
		}

		if start.len() == self.source.len() {
			return None;
		}

		Some(start.get(..start.len() - self.source.len()).unwrap())
	}

	pub fn strip_whitespace_and_comments(&mut self) {
		loop {
			// strip all leading whitespace, if any.
			self.take_while(Character::is_whitespace);

			// If we're not at the start of a comment, break out
			if self.advance_if('#').is_none() {
				break;
			}

			// eat a comment.
			self.take_while(|chr| chr != '\n');
		}
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	pub fn parse_program(mut self) -> Result<Value<'e>> {
		let ret = self.parse_expression()?;

		// If we forbid any trailing tokens, then see if we could have parsed anything else.
		#[cfg(feature = "compliance")]
		if self.env.flags().compliance.forbid_trailing_tokens
			&& !matches!(self.parse_expression().map_err(|e| e.kind), Err(ErrorKind::EmptySource))
		{
			return Err(self.error(ErrorKind::TrailingTokens));
		}

		Ok(ret)
	}

	pub fn strip_keyword_function(&mut self) {
		self.take_while(Character::is_upper);
	}

	pub fn strip_function(&mut self) {
		if self.peek().expect("strip function at eof").is_upper() {
			// If it's a keyword function, then take all keyword characters.
			self.take_while(Character::is_upper);
		} else {
			// otherwise, only take that character.
			self.advance();
		}
	}

	fn parse_grouped_expression(&mut self) -> Result<Value<'e>> {
		use ErrorKind::*;

		let start = self.line;

		match self.parse_expression() {
			Ok(val) => {
				self.strip_whitespace_and_comments();
				self.advance_if(')').and(Some(val)).ok_or_else(|| UnmatchedLeftParen.error(start))
			}
			Err(Error { kind: EmptySource, .. }) => Err(UnmatchedLeftParen.error(start)),
			Err(Error { kind: UnmatchedRightParen, .. }) => Err(DoesntEncloseExpression.error(start)),
			Err(other) => Err(other),
		}
	}

	pub fn parse_expression(&mut self) -> Result<Value<'e>> {
		let mut i = 0;
		while i < self.env.parsers().len() {
			if let Some(tmp) = self.env.parsers()[i].clone()(self)? {
				return Ok(tmp);
			}
			i += 1;
		}

		Err(
			self
				.error(self.peek().map(ErrorKind::UnknownTokenStart).unwrap_or(ErrorKind::EmptySource)),
		)
	}
}
