use super::ParseErrorKind;
use crate::options::Options;
use crate::parser::ParseError;
use crate::strings::StringSlice;
use crate::value::KString;
use crate::vm::Parser;

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

	// here for `=` function, and also `XLOCAL` and what have you
	pub fn parse_name<'e>(parser: &mut Parser<'_, 'e>) -> Result<Option<Self>, ParseError> {
		if !parser.peek().map_or(false, |c| c.is_lowercase() || c == '_') {
			return Ok(None);
		}

		let name = parser
			.take_while(|c| c.is_lowercase() || c.is_digit(10))
			.expect("we just checked for this");

		// i dont like this new_unvalidated. TODO: fix it.
		Self::new(StringSlice::new_unvalidated(name), parser.opts())
			.map_err(|err| parser.error(err))
			.map(Some)
	}
}
