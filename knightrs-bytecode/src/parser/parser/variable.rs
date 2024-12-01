use crate::strings::StringSlice;
use crate::value::KString;
use crate::vm::{ParseError, ParseErrorKind, Parseable, Parser};

pub struct Variable;

impl Variable {
	// here for `=` function, and also `XLOCAL` and what have you
	pub fn parse_name<'e>(parser: &mut Parser<'_, 'e>) -> Result<Option<&'e str>, ParseError> {
		if !parser.peek().map_or(false, |c| c.is_lowercase() || c == '_') {
			return Ok(None);
		}

		let name = parser
			.take_while(|c| c.is_lowercase() || c.is_digit(10))
			.expect("we just checked for this");

		#[cfg(feature = "compliance")]
		if parser.opts().compliance.variable_name_length
			&& name.len() <= crate::parser::VariableName::MAX_NAME_LEN
		{
			// i dont like this new_unvalidated. TODO: fix it.
			return Err(
				parser.error(ParseErrorKind::VariableNameTooLong(KString::new_unvalidated(name))),
			);
		}

		Ok(Some(name))
	}
}

// pub fn parse() {}
unsafe impl Parseable for Variable {
	fn parse(parser: &mut Parser<'_, '_>) -> Result<bool, ParseError> {
		let Some(name) = Self::parse_name(parser)? else {
			return Ok(false);
		};

		// TODO: ew, cloning the opts is icky as heck.
		let opts = (*parser.opts()).clone();
		parser.compiler().get_variable(StringSlice::new_unvalidated(name), &opts);
		Ok(true)
	}
}
