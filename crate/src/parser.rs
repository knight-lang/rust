use crate::{Ast, Boolean, Environment, Function, Number, Text, Value, Variable};
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Stream<I: Iterator<Item = char>> {
	iter: I,
	prev: Option<char>,
	rewound: bool,
	line: usize,
}

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
	UnterminatedQuote,

	/// A function was parsed, but one of its arguments was not able to be parsed.
	MissingFunctionArgument {
		/// The function whose argument is missing.
		func: char,

		/// The argument number.
		idx: usize,
	},

	/// An invalid character was encountered in a [`Text`](crate::Text) literal.
	InvalidString(crate::text::InvalidChar),

	/// A number literal was too large
	#[cfg(feature = "checked-overflow")]
	NumberLiteralOverflow,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "line {}: ", self.line)?;

		match self.kind {
			ParseErrorKind::NothingToParse => write!(f, "a token was expected."),
			ParseErrorKind::UnknownTokenStart(chr) => write!(f, "unknown token start {:?}.", chr),
			ParseErrorKind::UnterminatedQuote => write!(f, "unterminated quote encountered."),
			ParseErrorKind::MissingFunctionArgument { func, idx } => {
				write!(f, "missing argument {} for function {:?}.", idx, func)
			}
			ParseErrorKind::InvalidString(ref err) => write!(f, "{}", err),
			#[cfg(feature = "checked-overflow")]
			ParseErrorKind::NumberLiteralOverflow => write!(f, "integer literal overflowed max size"),
		}
	}
}

impl std::error::Error for ParseError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self.kind {
			ParseErrorKind::InvalidString(ref err) => Some(err),
			_ => None,
		}
	}
}

impl<I: Iterator<Item = char>> Stream<I> {
	pub fn new(iter: I) -> Self {
		Self {
			iter,
			prev: None,
			rewound: false,
			line: 1, // start on line 1
		}
	}

	pub fn rewind(&mut self) {
		assert!(!self.rewound);
		assert!(self.prev.is_some());

		self.rewound = true;

		if self.prev == Some('\n') {
			self.line -= 1;
		}
	}

	pub fn prev(&self) -> Option<char> {
		self.prev
	}

	pub fn peek(&mut self) -> Option<char> {
		let next = self.next();
		self.rewind();
		next
	}
}

impl<I: Iterator<Item = char>> Iterator for Stream<I> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		let next;

		if self.rewound {
			self.rewound = false;
			next = self.prev;
		} else {
			next = self.iter.next();

			if (next.is_some()) {
				self.prev = next;
			}
		}

		if next == Some('\n') {
			self.line += 1;
		}

		next
	}
}

fn is_whitespace(chr: char) -> bool {
	matches!(chr, ' ' | '\n' | '\r' | '\t' | '(' | ')' | '[' | ']' | '{' | '}' | ':')
}

impl<I: Iterator<Item = char>> Stream<I> {
	pub fn whitespace(&mut self) {
		if cfg!(debug_assertions) {
			match self.peek() {
				Some(chr) if is_whitespace(chr) => {}
				Some(other) => panic!("start character '{:?}' is not whitespace", other),
				None => panic!("encountered end of stream"),
			}
		}

		while let Some(chr) = self.next() {
			if !is_whitespace(chr) {
				self.rewind();
				break;
			}
		}
	}

	pub fn comment(&mut self) {
		debug_assert_eq!(self.peek(), Some('#'), "stream doesn't start with a '#'");

		for chr in self {
			if chr == '\n' {
				break;
			}
		}
	}

	pub fn number(&mut self) -> Result<Number, ParseError> {
		if cfg!(debug_assertions) {
			match self.peek() {
				Some('0'..='9') => {}
				Some(other) => panic!("start character '{:?}' is not a digit", other),
				None => panic!("encountered end of stream"),
			}
		}

		let mut number: Number = 0;

		while let Some(digit) = self.next() {
			if !digit.is_ascii_digit() {
				self.rewind();
				break;
			}

			cfg_if! {
				if #[cfg(feature="checked-overflow")] {
					number = number
						.checked_mul(10)
						.and_then(|num| num.checked_add((digit as u8 - b'0') as _))
						.ok_or_else(|| ParseError { line: self.line, kind: ParseErrorKind::NumberLiteralOverflow })?;
				} else {
					number = number.wrapping_mul(10).wrapping_add((digit as u8 - b'0') as _);
				}
			};
		}

		Ok(number)
	}

	pub fn variable(&mut self, env: &mut Environment<'_, '_, '_>) -> Variable {
		if cfg!(debug_assertions) {
			match self.peek() {
				Some('a'..='z') | Some('_') => {}
				Some(other) => panic!("start character '{:?}' is not a valid identifier start", other),
				None => panic!("encountered end of stream"),
			}
		}

		let mut ident = String::new();

		while let Some(chr) = self.next() {
			if chr.is_ascii_digit() || chr.is_ascii_lowercase() || chr == '_' {
				ident.push(chr);
			} else {
				self.rewind();
				break;
			}
		}

		env.get(&ident)
	}
	pub fn text(&mut self) -> Result<Text, ParseError> {
		let line = self.line;
		let quote = match self.next() {
			Some(quote @ '\'') | Some(quote @ '\"') => quote,
			Some(other) => panic!("character {:?} is not '\\'' or '\\\"'", other),
			None => panic!("encountered end of stream"),
		};

		let mut text = String::new();

		for chr in self {
			if chr == quote {
				return Text::try_from(text)
					.map_err(|err| ParseError { line, kind: ParseErrorKind::InvalidString(err) });
			}

			text.push(chr);
		}

		Err(ParseError { line, kind: ParseErrorKind::UnterminatedQuote })
	}

	pub fn boolean(&mut self) -> Boolean {
		let is_true = match self.next() {
			Some('T') => true,
			Some('F') => false,
			Some(other) => panic!("character {:?} is not 'T' or 'F'", other),
			None => panic!("encountered end of stream"),
		};

		self.strip_word();

		is_true
	}

	pub fn null(&mut self) {
		match self.next() {
			Some('N') => self.strip_word(),
			Some(other) => panic!("character {:?} is not 'N'", other),
			None => panic!("encountered end of stream"),
		}
	}

	pub fn function(
		&mut self,
		func: Function,
		env: &mut Environment<'_, '_, '_>,
	) -> Result<Value, ParseError> {
		let mut args = Vec::with_capacity(func.arity());
		let line = self.line;

		if func.name().is_ascii_uppercase() {
			self.strip_word();
		}

		for idx in 0..func.arity() {
			match self.parse(env) {
				Ok(value) => args.push(value),
				Err(ParseError { kind: ParseErrorKind::NothingToParse, .. }) => {
					return Err(ParseError {
						line,
						kind: ParseErrorKind::MissingFunctionArgument { func: func.name(), idx },
					})
				}
				Err(other) => return Err(other),
			}
		}

		// If a BLOCk _without_ a function as an argument is encountered, treat it as `: <arg>`.
		if cfg!(feature = "strict-block-return-value")
			&& func == crate::function::BLOCK_FUNCTION
			&& !matches!(args[0], Value::Ast(_))
		{
			args = vec![Value::Ast(Ast::new(crate::function::NOOP_FUNCTION, args.into_boxed_slice()))];
		}

		Ok(Value::Ast(Ast::new(func, args.into_boxed_slice())))
	}

	fn strip_word(&mut self) {
		while let Some(chr) = self.next() {
			if !matches!(chr, 'A'..='Z' | '_') {
				self.rewind();
				return;
			}
		}
	}

	pub fn parse(&mut self, env: &mut Environment<'_, '_, '_>) -> Result<Value, ParseError> {
		match self
			.peek()
			.ok_or_else(|| ParseError { line: self.line, kind: ParseErrorKind::NothingToParse })?
		{
			// note that this is ascii whitespace, as non-ascii characters are invalid.
			' ' | '\n' | '\r' | '\t' | '(' | ')' | '[' | ']' | '{' | '}' | ':' => {
				self.whitespace();
				self.parse(env)
			}

			// strip comments until eol.
			'#' => {
				self.comment();
				self.parse(env)
			}

			// only ascii digits may start a number.
			'0'..='9' => Ok(Value::Number(self.number()?)),

			// identifiers start only with lower-case digits or `_`.
			'a'..='z' | '_' => Ok(Value::Variable(self.variable(env))),

			'T' | 'F' => Ok(Value::Boolean(self.boolean())),

			'N' => {
				self.null();
				Ok(Value::Null)
			}

			// strings start with a single or double quote (and not `` ` ``).
			'\'' | '\"' => Ok(Value::Text(self.text()?)),

			chr => {
				if let Some(func) = Function::fetch(chr) {
					self.next();

					self.function(func, env)
				} else {
					Err(ParseError { line: self.line, kind: ParseErrorKind::UnknownTokenStart(chr) })
				}
			}
		}
	}
}

impl Value {
	/// Parses out a stream from the given `input` within the context of `env`.
	///
	/// This function simply calls [`parse`](Self::parse) with a char iterator over `input`; see it for more details.
	///
	/// # Errors
	/// This function returns any errors that [`parse`](Self::parse) returns.
	pub fn parse_str<S: AsRef<str>>(
		input: S,
		env: &mut Environment<'_, '_, '_>,
	) -> Result<Self, ParseError> {
		Self::parse(input.as_ref().chars(), env)
	}

	/// Parses out a stream from the given `input` within the context of `env`.
	///
	/// Note: Yes, technically this could be an iterator over `u8`, as the Knight specs clearly state that all source
	/// bytes are a subset of ASCII. However, we may want to support fun stuff like non-ASCII variables as an optional
	/// extension in the future. As such, `char` is required.
	///
	/// # Errors
	/// This function returns any errors that occur whilst parsing; See [`ParseError`]'s variants for what conditions can
	/// cause errors.
	///
	/// # See Also
	/// Section 1. within the Knight specs for parsing.
	pub fn parse<S: IntoIterator<Item = char>>(
		input: S,
		env: &mut Environment<'_, '_, '_>,
	) -> Result<Self, ParseError> {
		Stream::new(input.into_iter()).parse(env)
	}
}
