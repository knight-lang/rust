mod ast;
mod function;

#[cfg(feature = "check-parens")]
mod parens;

use super::VariableName;
use crate::parser::{
	source_location::ProgramSource, ParseError, ParseErrorKind, Parseable, SourceLocation,
};
use crate::program::{Compilable, Compiler, DeferredJump, JumpIndex, Program};
use crate::Gc;
use crate::{Environment, Options};
use std::path::Path;

pub struct Parser<'env, 'src, 'path, 'gc> {
	env: &'env mut Environment<'gc>,
	filename: ProgramSource<'path>,
	source: &'src str, // can't use `KnStr` b/c it has a length limit.
	compiler: Compiler<'src, 'path, 'gc>,
	lineno: usize,

	// Start is loop begin, vec is those to jump to loop end
	loops: Vec<(JumpIndex, Vec<DeferredJump>)>,
}

#[cfg(feature = "compliance")]
fn validate_source<'e, 'path>(
	source: &'e str,
	filename: ProgramSource<'path>,
	opts: &Options,
) -> Result<(), ParseError<'path>> {
	let Err(err) = opts.encoding.validate(source) else {
		return Ok(());
	};

	// figure out the line number; we can do btyes cause the encoding only fails in ascii and knight
	// 1 + because line numbering starts at 1
	let lineno = 1 + source.as_bytes().iter().take(err.position).filter(|&&c| c == b'\n').count();

	let whence = SourceLocation::new(filename, lineno);
	Err(ParseErrorKind::InvalidCharInEncoding(opts.encoding, err.character).error(whence))
}

impl<'env, 'src, 'path, 'gc> Parser<'env, 'src, 'path, 'gc> {
	pub fn new(
		env: &'env mut Environment<'gc>,
		filename: ProgramSource<'path>,
		source: &'src str,
	) -> Result<Self, ParseError<'path>> {
		#[cfg(feature = "compliance")]
		validate_source(source, filename, env.opts())?;

		Ok(Self {
			compiler: Compiler::new(SourceLocation::new(filename, 1), env.gc()),
			env,
			filename,
			source,
			lineno: 1,
			loops: Vec::new(),
		})
	}

	pub fn compiler(&mut self) -> &mut Compiler<'src, 'path, 'gc> {
		&mut self.compiler
	}

	pub fn opts(&self) -> &Options {
		self.env.opts()
	}

	pub fn gc(&self) -> &'gc Gc {
		self.env.gc()
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
			#[cfg(feature = "debugger")]
			self.compiler.record_source_location(self.location());
		}

		self.source = chars.as_str();
		Some(head)
	}

	/// Advance unequivocally.
	pub fn advance(&mut self) -> Option<char> {
		self.advance_if(|_| true)
	}

	/// Takes characters from while `func` returns true. `None` is returned if nothing was parsed.
	pub fn take_while<F>(&mut self, mut func: F) -> Option<&'src str>
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
	pub fn strip_whitespace_and_comments(&mut self) -> Option<&'src str> {
		let start = self.source;

		#[cfg(feature = "check-parens")]
		let check_parens = self.opts().check_parens;

		// TODO: when not in stacktrace mode, consider (, ), and : as whitespace
		loop {
			// strip all leading whitespace, if any.
			self.take_while(|c| {
				if c.is_whitespace() {
					return true;
				}

				#[cfg(feature = "check-parens")]
				if check_parens {
					// TODO: strict compliance
					return false;
				}

				matches!(c, '(' | ')' | ':')
			});

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
	pub fn location(&self) -> SourceLocation<'path> {
		SourceLocation::new(self.filename.clone(), self.lineno)
	}

	/// Removes the remainder of a keyword function.
	pub fn strip_keyword_function(&mut self) -> Option<&'src str> {
		self.take_while(|c| c.is_uppercase() || c == '_')
	}

	/// Creates an error at the current source code position.
	#[must_use]
	pub fn error(&self, kind: ParseErrorKind) -> ParseError<'path> {
		kind.error(self.location())
	}

	/// Parses a whole program, returning a [`Value`] corresponding to its ast.
	///
	/// This will return an [`ErrorKind::TrailingTokens`] if [`forbid_trailing_tokens`](
	/// crate::env::flags::Compliance::forbid_trailing_tokens) is set.
	pub fn parse_program(mut self) -> Result<Program<'src, 'path, 'gc>, ParseError<'path>> {
		self.parse_expression()?;

		// If we forbid any trailing tokens, then see if we could have parsed anything else.
		#[cfg(feature = "compliance")]
		if self.env.opts().compliance.forbid_trailing_tokens
			&& !matches!(self.parse_expression().map_err(|e| e.kind), Err(ParseErrorKind::EmptySource))
		{
			return Err(self.error(ParseErrorKind::TrailingTokens));
		}

		// SAFETY: this program ensures that things are built properly
		Ok(unsafe { self.compiler.build() })
	}

	/// Parses a single expression and returns it.
	pub fn parse_expression(&mut self) -> Result<(), ParseError<'path>> {
		self.strip_whitespace_and_comments();

		if let Some(x) = crate::value::Integer::parse(self)? {
			return x.compile(&mut self.compiler, &self.env.opts());
		}
		if let Some(x) = crate::value::Boolean::parse(self)? {
			return x.compile(&mut self.compiler, &self.env.opts());
		}
		if let Some(x) = crate::value::Null::parse(self)? {
			return x.compile(&mut self.compiler, &self.env.opts());
		}
		if let Some(x) = crate::value::List::parse(self)? {
			return x.compile(&mut self.compiler, &self.env.opts());
		}
		if let Some(x) = crate::value::KnString::parse(self)? {
			return x.compile(&mut self.compiler, &self.env.opts());
		}
		if let Some(x) = VariableName::parse(self)? {
			return x.compile(&mut self.compiler, &self.env.opts());
		}

		#[cfg(feature = "check-parens")]
		if self.env.opts().check_parens && parens::parse_parens(self)? {
			return Ok(());
		}

		if function::Function::parse(self)? {
			return Ok(());
		}

		let chr = self.peek().ok_or_else(|| self.error(ParseErrorKind::EmptySource))?;
		Err(self.error(ParseErrorKind::UnknownTokenStart(chr)))
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
