//! Parsing Knight code.

use crate::containers::{MaybeSendSync, RefCount};
use crate::env::{Environment, Flags};
use crate::value::text::TextSlice;
use crate::value::{integer::IntType, Value};
use std::fmt::{self, Display, Formatter};

mod blank;
mod grouped_expression;
#[cfg(feature = "extensions")]
mod list_literal;
pub use blank::Blank;
pub use grouped_expression::GroupedExpression;
#[cfg(feature = "extensions")]
pub use list_literal::ListLiteral;

/// A type that handles parsing source code.
#[must_use]
pub struct Parser<'s, 'e, I> {
	source: &'s TextSlice,
	env: &'s mut Environment<'e, I>,
	line: usize,
}

/// A trait that indicates that something can be parsed.
pub trait Parsable<I>: Sized {
	/// The type that's being parsed.
	type Output;

	/// Attempt to parse an `Output` from the `parser`.
	///
	/// - If an `Output` was successfully parsed, then return `Ok(Some(...))`.
	/// - If there's nothing applicable to parse from `parser`, then `Ok(None)` should be returned.
	/// - If parsing should be restarted from the top (e.g. the [`Blank`] parser removing
	///   whitespace), then [`ErrorKind::RestartParsing`] should be returned.
	/// - If there's an issue when parsing (such as missing a closing quote), an [`Error`] should be
	///   returned.
	fn parse(parser: &mut Parser<'_, '_, I>) -> Result<Option<Self::Output>>;

	/// A convenience function that generates things you can stick into [`env::Builder::parsers`](
	/// crate::env::Builder::parsers).
	fn parse_fn() -> ParseFn<I>
	where
		Value<I>: From<Self::Output>,
		I: IntType,
	{
		RefCount::new(|parser| Ok(Self::parse(parser)?.map(Value::from)))
	}
}

/// A type that can parse things.
pub type ParseFn<I> = RefCount<dyn ParseFn_<I>>;

/// A Trait that indicates something is able to be parsed.
pub trait ParseFn_<I: IntType>:
	Fn(&mut Parser<'_, '_, I>) -> Result<Option<Value<I>>> + MaybeSendSync
{
}

impl<T, I> ParseFn_<I> for T
where
	I: IntType,
	T: Fn(&mut Parser<'_, '_, I>) -> Result<Option<Value<I>>> + MaybeSendSync,
{
}

// Gets the default list of parsers. (We don't use the `_flags` field currently, but it's there
// in case we want it for extensions later.)
pub(crate) fn default<I>(_flags: &Flags) -> Vec<ParseFn<I>>
where
	I: IntType,
{
	macro_rules! parsers {
		($($(#[$meta:meta])* $ty:ty),* $(,)?) => {
			vec![$($(#[$meta])* <$ty>::parse_fn()),*]
		};
	}

	parsers![
		Blank,
		GroupedExpression,
		crate::value::Integer<I>,
		crate::value::Text,
		crate::env::Variable< I>,
		crate::value::Boolean,
		crate::value::Null,
		crate::value::List< I>,
		crate::ast::Ast< I>,

		#[cfg(feature = "extensions")]
		ListLiteral< I>
	]
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
	/// Indicates that while something was parsed, parsing should be restarted regardless.
	/// Used within whitespace and comments.
	RestartParsing,

	/// End of stream was reached before a token could be parsed.
	EmptySource,

	/// An unrecognized character was encountered.
	UnknownTokenStart(char),

	/// A starting quote was found without an associated ending quote.
	UnterminatedText {
		/// The starting character of the quote (ie either `'` or `"`)
		quote: char,
	},

	/// A function name was parsed, but an argument of its was missing.
	MissingArgument {
		/// The name of the function whose argument is missing.
		name: String,

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

	/// A variable name wasn't valid for some reason
	///
	/// This is only returned when the `verify-variable-names` is enabled.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	IllegalVariableName(crate::env::variable::IllegalVariableName),

	/// The source file wasn't exactly one expression.
	///
	/// This is only returned when `forbid-trailing-tokens` is enabled.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	TrailingTokens,

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	/// An unknown extension name was encountered.
	UnknownExtensionFunction(String),

	/// An error which doesn't fit into one of the other categories.
	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	Custom(Box<dyn std::error::Error + Send + Sync>),
}

impl Display for ErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::RestartParsing => write!(f, "<restart parsing>"),
			Self::EmptySource => write!(f, "an empty source string was encountered"),
			Self::UnknownTokenStart(chr) => write!(f, "unknown token start {chr:?}"),
			Self::UnterminatedText { quote } => write!(f, "unterminated `{quote}` text"),
			Self::MissingArgument { name, index } => {
				write!(f, "missing argument {index} for function {name:?}")
			}
			Self::IntegerLiteralOverflow => write!(f, "integer literal overflowed max size"),

			Self::UnmatchedLeftParen => write!(f, "an unmatched `(` was encountered"),
			Self::UnmatchedRightParen => write!(f, "an unmatched `)` was encountered"),
			Self::DoesntEncloseExpression => write!(f, "parens dont enclose an expression"),

			#[cfg(feature = "compliance")]
			Self::IllegalVariableName(ref err) => Display::fmt(&err, f),

			#[cfg(feature = "compliance")]
			Self::TrailingTokens => write!(f, "trailing tokens encountered"),

			#[cfg(feature = "extensions")]
			Self::UnknownExtensionFunction(ref name) => write!(f, "unknown extension {name}"),

			#[cfg(feature = "extensions")]
			Self::Custom(err) => Display::fmt(err, f),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "line {}: {}", self.line, self.kind)
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		match self.kind {
			#[cfg(feature = "compliance")]
			ErrorKind::IllegalVariableName(ref err) => Some(&*err),

			#[cfg(feature = "extensions")]
			ErrorKind::Custom(ref err) => Some(&**err),

			_ => None,
		}
	}
}

impl ErrorKind {
	/// Helper function to create a new [`Error`].
	pub const fn error(self, line: usize) -> Error {
		Error { line, kind: self }
	}
}

/// Helper trait for [`Praser::advance_if`].
pub trait AdvanceIfCondition {
	/// Checks to see whether we should advance past `chr`.
	fn should_advance(self, chr: char) -> bool;
}

impl<T: FnOnce(char) -> bool> AdvanceIfCondition for T {
	fn should_advance(self, chr: char) -> bool {
		self(chr)
	}
}

impl AdvanceIfCondition for char {
	fn should_advance(self, chr: char) -> bool {
		self == chr
	}
}

impl<'s, 'e, I> Parser<'s, 'e, I> {
	/// Create a new `Parser` from the given source.
	#[must_use]
	pub fn new(source: &'s TextSlice, env: &'s mut Environment<'e, I>) -> Self {
		Self { source, line: 1, env }
	}

	/// Gets the current line number.
	#[must_use]
	pub fn line(&self) -> usize {
		self.line
	}

	/// Gets the environment.
	#[must_use]
	pub fn env(&mut self) -> &mut Environment<'e, I> {
		self.env
	}

	/// Creates an error at the current source code position.
	#[must_use]
	pub fn error(&self, kind: ErrorKind) -> Error {
		kind.error(self.line)
	}

	/// Gets, without consuming, the next character (if it exists).
	#[must_use = "peeking doesn't advance the parser"]
	pub fn peek(&self) -> Option<char> {
		self.source.head()
	}

	/// Gets, and advances past, the next character if `cond` matches.
	pub fn advance_if<F>(&mut self, cond: F) -> Option<char>
	where
		F: AdvanceIfCondition,
	{
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

	/// Advance unequivocally.
	pub fn advance(&mut self) -> Option<char> {
		self.advance_if(|_| true)
	}

	/// Takes characters from while `func` returns true. `None` is returned if nothing was parsed.
	pub fn take_while<F>(&mut self, mut func: F) -> Option<&'s TextSlice>
	where
		F: FnMut(char) -> bool,
	{
		let start = self.source;

		while self.peek().map_or(false, &mut func) {
			self.advance();
		}

		if start.len() == self.source.len() {
			return None;
		}

		Some(start.get(..start.len() - self.source.len()).unwrap())
	}
	// }

	// impl<'s,  I: Encoding> Parser<'s,  I> {
	/// Removes leading whitespace and comments, returning whether anything _was_ stripped.
	pub fn strip_whitespace_and_comments(&mut self) -> Option<&'s TextSlice> {
		let start = self.source;

		loop {
			// strip all leading whitespace, if any.
			self.take_while(|c| c.is_whitespace() || c == ':');

			// If we're not at the start of a comment, break out
			if self.advance_if('#').is_none() {
				break;
			}

			// Eat a comment.
			self.take_while(|chr| chr != '\n');
		}

		if start.len() == self.source.len() {
			return None;
		}

		Some(start.get(..start.len() - self.source.len()).unwrap())
	}

	/// Removes the remainder of a keyword function.
	pub fn strip_keyword_function(&mut self) -> Option<&'s TextSlice> {
		self.take_while(|c| c.is_uppercase() || c == '_')
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	///
	/// This will return an [`ErrorKind::TrailingTokens`] if [`forbid_trailing_tokens`](
	/// crate::env::flags::Compliance::forbid_trailing_tokens) is set.
	pub fn parse_program(mut self) -> Result<Value<I>>
	where
		I: IntType,
	{
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

	/// Parses a single expression and returns it.
	///
	/// This goes through its [environment's parsers](Environment::parsers) one by one, returning
	/// the first value that returned `Ok(Some(...))`
	pub fn parse_expression(&mut self) -> Result<Value<I>>
	where
		I: IntType,
	{
		let mut i = 0;
		// This is quite janky, we should fix it up.
		while i < self.env.parsers().len() {
			match self.env.parsers()[i].clone()(self) {
				Err(Error { kind: ErrorKind::RestartParsing, .. }) => i = 0,
				Err(err) => return Err(err),
				Ok(Some(tmp)) => return Ok(tmp),
				Ok(None) => i += 1,
			}
		}

		Err(
			self.error(
				self
					.peek()
					.map(char::from)
					.map(ErrorKind::UnknownTokenStart)
					.unwrap_or(ErrorKind::EmptySource),
			),
		)
	}
}
