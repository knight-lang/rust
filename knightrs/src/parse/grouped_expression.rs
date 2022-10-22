use super::*;
use ErrorKind::{DoesntEncloseExpression, EmptySource, UnmatchedLeftParen, UnmatchedRightParen};

/// A [`Parsable`] that ensures that parens are matched.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupedExpression;

impl Parsable for GroupedExpression {
	type Output = Value;

	fn parse(parser: &mut Parser<'_, '_>) -> Result<Option<Self::Output>> {
		if parser.advance_if(')').is_some() {
			return Err(parser.error(UnmatchedRightParen));
		}

		if parser.advance_if('(').is_none() {
			return Ok(None);
		}

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
