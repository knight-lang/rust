use super::{ParseErrorKind, Parseable, SourceLocation};
use crate::options::Options;
use crate::parser::{ParseError, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::KnStr;
use std::fmt::{self, Display, Formatter};

/// The name of a variable within Knight.
///
/// It's scoped to `'src`, but can be made static via [`VariableName::become_owned`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableName<'src>(crate::container::RcOrRef<'src, KnStr>);

impl<'src> VariableName<'src> {
	/// The maximum length that variables can have when compliance checking is disabled.
	pub const MAX_NAME_LEN: usize = 127;

	/// Creates a new [`VariableName`] from the given `name`.
	///
	/// # Errors
	/// If [`opts.compliance.variable_name_length`](Options::Compliance::variable_name_lemgth) is
	/// enabled, and `name`'s length is longer than [`MAX_NAME_LEN`](VariableName::MAX_NAME_LEN),
	/// [`ParseErrorKind::VariableNameTooLong`] will be returned
	pub fn new(name: &'src KnStr, opts: &Options) -> Result<Self, ParseErrorKind> {
		#[cfg(feature = "compliance")]
		if opts.compliance.variable_name_length && Self::MAX_NAME_LEN < name.len() {
			return Err(ParseErrorKind::VariableNameTooLong(name.to_owned().to_string()));
		}

		Ok(Self(name.into()))
	}

	/// Same as [`VariableName::new`], except `name` must always be at most [`Self::MAX_NAME_LEN`].
	///
	/// # Compliance
	/// It's a Knight compliance violation if `name`'s length is longer than [`Self::MAX_NAME_LEN`].
	pub fn new_unvalidated(name: &'src KnStr) -> Self {
		debug_assert!(name.len() <= Self::MAX_NAME_LEN);

		Self(name.into())
	}

	/// Converts `self` into an owned version of a [`VariableName`].
	pub fn become_owned(self) -> VariableName<'static> {
		VariableName(self.0.into_owned_a().into())
	}
}

impl<'src, 'path> Parseable<'src, 'path, '_> for VariableName<'src> {
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
			.expect("at least one element should exist, as we checked for lower || '_' earlier");

		Self::new(KnStr::new_unvalidated(name), parser.opts())
			.map_err(|err| parser.error(err))
			.map(|name| Some((name, start)))
	}
}

unsafe impl<'src, 'path> Compilable<'src, 'path, '_>
	for (VariableName<'src>, SourceLocation<'path>)
{
	fn compile(
		self,
		compiler: &mut Compiler<'src, '_, '_>,
		opts: &Options,
	) -> Result<(), crate::parser::ParseError<'path>> {
		compiler.get_variable(self.0, opts).map_err(|err| err.error(self.1))
	}
}

impl Display for VariableName<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		// TODO: remove `as_str` if we ever impl display
		Display::fmt(&self.0.as_str(), f)
	}
}
