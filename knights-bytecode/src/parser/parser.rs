mod function;
mod parens;
mod variable;

use crate::{container::RefCount, options::Options, vm::ParseErrorKind, Value};
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

use crate::{
	strings::{StringError, StringSlice},
	Environment,
};

use crate::vm::program::{DeferredJump, JumpIndex};
use crate::vm::{Builder, ParseError, Program};

#[derive(Debug, Clone, Default)] // DELETEME: default, just for testing stackframes
pub struct SourceLocation {
	filename: Option<RefCount<Path>>,
	line: usize,
}

impl SourceLocation {
	// this tuple is a huge hack. maybe when i remove it i can also remove `'filename`
	pub fn error(self, kind: ParseErrorKind) -> ParseError {
		ParseError { whence: self, kind }
	}
}

impl Display for SourceLocation {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if let Some(ref filename) = self.filename {
			write!(f, "{}:{}", filename.display(), self.line)
		} else {
			write!(f, "<expr>:{}", self.line)
		}
	}
}

// safety: cannot do invalid things with the builder.
pub unsafe trait Parseable {
	fn parse(parser: &mut Parser<'_, '_>) -> Result<bool, ParseError>;
}

pub struct Parser<'env, 'expr> {
	env: &'env mut Environment,
	filename: Option<RefCount<Path>>, // TODO: dont use refcount
	source: &'expr str,               // can't use `StringSlice` b/c it has a length limit.
	builder: Builder,
	lineno: usize,

	// Start is loop begin, vec is those to jump to loop end
	loops: Vec<(JumpIndex, Vec<DeferredJump>)>,
}

fn validate_source<'e>(
	source: &'e str,
	filename: &Option<RefCount<Path>>,
	opts: &Options,
) -> Result<(), ParseError> {
	let Err(err) = opts.encoding.validate(source) else {
		return Ok(());
	};

	// figure out the line number; we can do btyes cause the encoding only fails in ascii and knight
	// 1 + because line numbering starts at 1
	let lineno = 1 + source.as_bytes().iter().take(err.position).filter(|&&c| c == b'\n').count();

	let whence = SourceLocation { filename: filename.clone(), line: lineno };
	Err(ParseErrorKind::InvalidCharInEncoding(opts.encoding, err.character).error(whence))
}

impl<'env, 'expr> Parser<'env, 'expr> {
	pub fn new(
		env: &'env mut Environment,
		filename: Option<&Path>,
		source: &'expr str,
	) -> Result<Self, ParseError> {
		let filename = filename.map(|c| c.to_owned().into());

		#[cfg(feature = "compliance")]
		validate_source(source, &filename, env.opts())?;

		Ok(Self { env, filename, source, builder: Builder::default(), lineno: 1, loops: Vec::new() })
	}

	pub fn builder(&mut self) -> &mut Builder {
		&mut self.builder
	}

	pub fn opts(&self) -> &Options {
		self.env.opts()
	}

	pub fn peek(&self) -> Option<char> {
		self.source.chars().next()
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
			self.lineno += 1;
		}

		self.source = chars.as_str();
		Some(head)
	}

	/// Advance unequivocally.
	pub fn advance(&mut self) -> Option<char> {
		self.advance_if(|_| true)
	}

	/// Takes characters from while `func` returns true. `None` is returned if nothing was parsed.
	pub fn take_while<F>(&mut self, mut func: F) -> Option<&'expr str>
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

	/// Removes leading whitespace and comments, returning whether anything _was_ stripped.
	pub fn strip_whitespace_and_comments(&mut self) -> Option<&'expr str> {
		let start = self.source;

		loop {
			// strip all leading whitespace, if any.
			self.take_while(|c| c.is_whitespace() || c == ':'); // TODO: THIS WON'T HANDLE `(:)` PROPERLY!

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

	// ick,
	pub fn location(&self) -> SourceLocation {
		SourceLocation { filename: self.filename.clone(), line: self.lineno }
	}

	/// Removes the remainder of a keyword function.
	pub fn strip_keyword_function(&mut self) -> Option<&'expr str> {
		self.take_while(|c| c.is_uppercase() || c == '_')
	}

	/// Creates an error at the current source code position.
	#[must_use]
	pub fn error(&self, kind: ParseErrorKind) -> ParseError {
		kind.error(self.location())
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	///
	/// This will return an [`ErrorKind::TrailingTokens`] if [`forbid_trailing_tokens`](
	/// crate::env::flags::Compliance::forbid_trailing_tokens) is set.
	pub fn parse_program(mut self) -> Result<Program, ParseError> {
		self.parse_expression()?;

		// If we forbid any trailing tokens, then see if we could have parsed anything else.
		#[cfg(feature = "compliance")]
		if self.env.opts().compliance.forbid_trailing_tokens
			&& !matches!(self.parse_expression().map_err(|e| e.kind), Err(ParseErrorKind::EmptySource))
		{
			return Err(self.error(ParseErrorKind::TrailingTokens));
		}

		// SAFETY: this program ensures that things are built properly
		Ok(unsafe { self.builder.build() })
	}

	/// Parses a single expression and returns it.
	pub fn parse_expression(&mut self) -> Result<(), ParseError> {
		self.strip_whitespace_and_comments();

		crate::value::Integer::parse(self)? && return Ok(());
		crate::value::Boolean::parse(self)? && return Ok(());
		crate::value::KString::parse(self)? && return Ok(());
		crate::value::Null::parse(self)? && return Ok(());
		crate::value::List::parse(self)? && return Ok(());
		variable::Variable::parse(self)? && return Ok(());
		function::Function::parse(self)? && return Ok(());
		// todo: parens

		let chr = self.peek().ok_or_else(|| self.error(ParseErrorKind::EmptySource))?;

		match chr {
			_ if chr == '#' || chr.is_whitespace() => unreachable!("<already handled>"),
			'(' | ')' => todo!(),

			_ => Err(self.error(ParseErrorKind::UnknownTokenStart(chr))),
		}
	}

	fn parse_variable(&mut self) -> Result<(), ParseError> {
		todo!() // todo: check for int overflow
	}

	fn parse_string(&mut self) -> Result<(), ParseError> {
		todo!() // todo: check for int overflow
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
