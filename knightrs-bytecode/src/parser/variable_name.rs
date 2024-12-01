use super::{ParseErrorKind, Parseable, SourceLocation};
use crate::options::Options;
use crate::parser::{ParseError, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::StringSlice;
use crate::value::KString;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableName(KString);

impl VariableName {
	#[cfg(feature = "compliance")]
	pub const MAX_NAME_LEN: usize = 127;

	pub fn new(name: &StringSlice, opts: &Options) -> Result<Self, ParseErrorKind> {
		#[cfg(feature = "compliance")]
		if opts.compliance.variable_name_length && Self::MAX_NAME_LEN < name.len() {
			return Err(ParseErrorKind::VariableNameTooLong(name.to_owned()));
		}

		Ok(Self(name.to_owned()))
	}
}

impl Parseable for VariableName {
	type Output = (Self, SourceLocation);

	fn parse(parser: &mut Parser<'_, '_>) -> Result<Option<Self::Output>, ParseError> {
		if !parser.peek().map_or(false, |c| c.is_lowercase() || c == '_') {
			return Ok(None);
		}

		let start = parser.location();

		let name = parser
			.take_while(|c| c.is_lowercase() || c.is_digit(10))
			.expect("we just checked for this");

		// i dont like this new_unvalidated. TODO: fix it.
		Self::new(StringSlice::new_unvalidated(name), parser.opts())
			.map_err(|err| parser.error(err))
			.map(|name| Some((name, start)))
	}
}

unsafe impl Compilable for (VariableName, SourceLocation) {
	fn compile(
		self,
		compiler: &mut Compiler,
		opts: &Options,
	) -> Result<(), crate::parser::ParseError> {
		compiler.get_variable(self.0, opts).map_err(|err| self.1.error(err))
	}
}

impl Display for VariableName {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		// TODO: remove `as_Str` if we ever impl display
		Display::fmt(&self.0.as_str(), f)
	}
}
