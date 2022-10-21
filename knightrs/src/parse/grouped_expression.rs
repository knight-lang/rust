use super::*;

/// A [`Parsable`] that ensures that parens are matched.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupedExpression;

impl<I: IntType, E: Encoding> Parsable<I, E> for GroupedExpression {
	type Output = Value<I, E>;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> Result<Option<Self::Output>> {
		use ErrorKind::{
			DoesntEncloseExpression, EmptySource, UnmatchedLeftParen, UnmatchedRightParen,
		};

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
