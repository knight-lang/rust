use crate::parser::ParseError;
use crate::strings::StringSlice;

use crate::options::Options;
use crate::value::KString;

use super::ParseErrorKind;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableName(KString);

impl VariableName {
	#[cfg(feature = "compliance")]
	pub const MAX_NAME_LEN: usize = 127;

	pub fn new(name: &StringSlice, opts: &Options) -> Result<Self, ParseErrorKind> {
		#[cfg(feature = "compliance")]
		if opts.compliance.variable_name_length && Self::MAX_NAME_LEN < name.len() {
			// i dont like this new_unvalidated. TODO: fix it.
			return Err(ParseErrorKind::VariableNameTooLong(name.to_owned()));
		}

		Ok(Self(name.to_owned()))
	}
}
