use super::{ParseErrorKind, Parseable, SourceLocation};
use crate::options::Options;
use crate::parser::{ParseError, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::KnStr;
use crate::value::KnString;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableName<'src>(crate::container::RcOrRef<'src, KnStr>);

impl<'src> VariableName<'src> {
	pub const MAX_NAME_LEN: usize = 127;

	/// Caller must ensure that the variable name is always <= MAX_NAME_LEN.
	pub fn new_unvalidated(name: &'src KnStr) -> Self {
		debug_assert!(name.len() <= Self::MAX_NAME_LEN);

		Self(name.into())
	}

	pub fn new(name: &'src KnStr, opts: &Options) -> Result<Self, ParseErrorKind> {
		#[cfg(feature = "compliance")]
		if opts.compliance.variable_name_length && Self::MAX_NAME_LEN < name.len() {
			return Err(ParseErrorKind::VariableNameTooLong(name.to_owned()));
		}

		Ok(Self(name.into()))
	}

	pub fn become_owned(self) -> VariableName<'static> {
		VariableName(self.0.into_owned_a().into())
	}
}

impl<'src, 'path> Parseable<'src, 'path> for VariableName<'src> {
	type Output = (Self, SourceLocation<'path>);

	fn parse(
		parser: &mut Parser<'_, 'src, 'path, '_>,
	) -> Result<Option<Self::Output>, ParseError<'path>> {
		if !parser.peek().map_or(false, |c| c.is_lowercase() || c == '_') {
			return Ok(None);
		}

		let start = parser.location();

		let name = parser
			.take_while(|c| c.is_lowercase() || c.is_digit(10) || c == '_')
			.expect("we just checked for this");

		// i dont like this new_unvalidated. TODO: fix it.
		Self::new(KnStr::new_unvalidated(name), parser.opts())
			.map_err(|err| parser.error(err))
			.map(|name| Some((name, start)))
	}
}

unsafe impl<'src, 'path> Compilable<'src, 'path> for (VariableName<'src>, SourceLocation<'path>) {
	fn compile(
		self,
		compiler: &mut Compiler<'src, '_>,
		opts: &Options,
	) -> Result<(), crate::parser::ParseError<'path>> {
		compiler.get_variable(self.0, opts).map_err(|err| self.1.error(err))
	}
}

impl Display for VariableName<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		// TODO: remove `as_Str` if we ever impl display
		Display::fmt(&self.0.as_str(), f)
	}
}
