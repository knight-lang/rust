use crate::parser::{ParseError, ParseErrorKind, Parser};

pub fn parse_parens<'path>(
	parser: &mut Parser<'_, '_, 'path, '_>,
) -> Result<bool, ParseError<'path>> {
	// If we have a `)`, that means it's a random `)` in the source.
	if parser.advance_if(')').is_some() {
		return Err(parser.error(ParseErrorKind::UnmatchedClosingParen));
	}

	// If we don't have a `(`, then we aren't parsing parens/
	if parser.advance_if('(').is_none() {
		return Ok(false);
	}

	let start = parser.location();
	parser.parse_expression()?;

	//
	parser.strip_whitespace_and_comments();
	if parser.advance_if(')').is_none() {
		return Err(start.error(ParseErrorKind::MissingClosingParen));
	}

	return Ok(true);
}
