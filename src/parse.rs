use crate::containers::{MaybeSendSync, RefCount};
use crate::env::Variable;
use crate::text::{Character, Text, TextSlice};
use crate::value::{integer::IntType, Integer, List, Value};
use crate::{Ast, Environment};
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
pub struct Parser<'s, 'e, I: IntType> {
	source: &'s TextSlice,
	env: &'s mut Environment<'e, I>,
	line: usize,
}

/// A trait that indicates that something can be parsed.
pub trait Parsable<'e, I: IntType>: Sized {
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
	fn parse(parser: &mut Parser<'_, 'e, I>) -> Result<Option<Self::Output>>;

	/// A convenience function that generates things you can stick into [`env::Builder::parsers`](
	/// crate::env::Builder::parsers).
	fn parse_fn() -> RefCount<dyn ParseFn<'e, I>>
	where
		Value<'e, I>: From<Self::Output>,
	{
		RefCount::from(Box::new(|parser: &mut Parser<'_, 'e, I>| {
			Ok(Self::parse(parser)?.map(Value::from))
		}) as Box<_>)
	}
}

/// A Trait that indicates something is able to be parsed.
pub trait ParseFn<'e, I: IntType>:
	Fn(&mut Parser<'_, 'e, I>) -> Result<Option<Value<'e, I>>> + MaybeSendSync
{
}

impl<'e, I, T> ParseFn<'e, I> for T
where
	I: IntType,
	T: Fn(&mut Parser<'_, 'e, I>) -> Result<Option<Value<'e, I>>> + MaybeSendSync,
{
}

// Gets the default list of parsers. (We don't use the `_flags` field currently, but it's there
// in case we want it for extensions later.)
pub(crate) fn default<'e, I: IntType + 'e>(
	_flags: &crate::env::Flags,
) -> Vec<RefCount<dyn ParseFn<'e, I>>> {
	macro_rules! parsers {
		($($(#[$meta:meta])* $ty:ty),* $(,)?) => {
			vec![$($(#[$meta])* <$ty>::parse_fn()),*]
		};
	}

	parsers![
		Blank,
		GroupedExpression,
		Integer<I>,
		Text,
		Variable<'e, I>,
		crate::value::Boolean,
		crate::value::Null,
		List<'e, I>,
		Ast<'e, I>,
		#[cfg(feature = "extensions")]
		ListLiteral<'e, I>
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

	/// A variable name wasn't valid for some reason
	///
	/// This is only returned when the `verify-variable-names` is enabled.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	IllegalVariableName(crate::env::IllegalVariableName),

	/// The source file wasn't exactly one expression.
	///
	/// This is only returned when `forbid-trailing-tokens` is enabled.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	TrailingTokens,

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	/// An unknown extension name was encountered.
	UnknownExtensionFunction(Text),

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
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

impl<'s, 'e, I: IntType> Parser<'s, 'e, I> {
	/// Create a new `Parser` from the given source.
	pub fn new(source: &'s TextSlice, env: &'s mut Environment<'e, I>) -> Self {
		Self { source, line: 1, env }
	}

	pub fn line(&self) -> usize {
		self.line
	}

	pub fn env(&mut self) -> &mut Environment<'e, I> {
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

	pub fn take_while<F: FnMut(Character, &crate::env::Flags) -> bool>(
		&mut self,
		mut func: F,
	) -> Option<&'s TextSlice> {
		let start = self.source;

		while self.peek().map_or(false, |chr| func(chr, self.env.flags())) {
			self.advance();
		}

		if start.len() == self.source.len() {
			return None;
		}

		Some(start.get(..start.len() - self.source.len()).unwrap())
	}

	pub fn strip_whitespace_and_comments(&mut self) -> bool {
		let mut anything_stripped = false;
		loop {
			// strip all leading whitespace, if any.
			anything_stripped |= self.take_while(Character::is_whitespace).is_some();

			// If we're not at the start of a comment, break out
			if self.advance_if('#').is_none() {
				return anything_stripped;
			}

			// eat a comment.
			self.take_while(|chr, _| chr != '\n');
			anything_stripped = true;
		}
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	pub fn parse_program(mut self) -> Result<Value<'e, I>> {
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
		if self.peek().expect("strip function at eof").is_upper(self.env.flags()) {
			// If it's a keyword function, then take all keyword characters.
			self.take_while(Character::is_upper);
		} else {
			// otherwise, only take that character.
			self.advance();
		}
	}

	pub fn parse_expression(&mut self) -> Result<Value<'e, I>> {
		let mut i = 0;
		while i < self.env.parsers().len() {
			match self.env.parsers()[i].clone()(self) {
				Err(Error { kind: ErrorKind::RestartParsing, .. }) => i = 0,
				Err(err) => return Err(err),
				Ok(Some(tmp)) => return Ok(tmp),
				Ok(None) => i += 1,
			}
		}

		Err(
			self
				.error(self.peek().map(ErrorKind::UnknownTokenStart).unwrap_or(ErrorKind::EmptySource)),
		)
	}
}
