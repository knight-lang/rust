use crate::parser::{ParseError, ParseErrorKind, Parser};

pub fn parse_parens(parser: &mut Parser<'_, '_>) -> Result<bool, ParseError> {
	if parser.advance_if(')').is_some() {
		return Err(parser.error(ParseErrorKind::UnmatchedClosingParen));
	}

	if parser.advance_if('(').is_none() {
		return Ok(false);
	}

	let start = parser.location();
	parser.parse_expression()?;

	parser.strip_whitespace_and_comments();
	if parser.advance_if(')').is_none() {
		return Err(parser.error(ParseErrorKind::MissingClosingParen(start)));
	}

	return Ok(true);
}