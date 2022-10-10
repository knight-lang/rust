use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupedExpression;

impl<'e> Parsable<'e> for GroupedExpression {
	type Output = Value<'e>;

	fn parse(parser: &mut Parser<'_, 'e>) -> Result<Option<Self::Output>> {
		if parser.advance_if(')').is_some() {
			return Err(parser.error(ErrorKind::UnmatchedRightParen));
		}

		if parser.advance_if('(').is_none() {
			return Ok(None);
		}

		use ErrorKind::*;

		let start = parser.line;

		match parser.parse_expression() {
			Ok(val) => {
				parser.strip_whitespace_and_comments();
				parser.advance_if(')').ok_or_else(|| UnmatchedLeftParen.error(start)).and(Ok(Some(val)))
			}
			Err(Error { kind: EmptySource, .. }) => Err(UnmatchedLeftParen.error(start)),
			Err(Error { kind: UnmatchedRightParen, .. }) => Err(DoesntEncloseExpression.error(start)),
			Err(err) => Err(err),
		}
	}
}
