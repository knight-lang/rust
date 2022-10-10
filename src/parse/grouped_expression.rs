use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupedExpression;

impl<'e> Parsable<'e> for GroupedExpression {
	type Output = Value<'e>;

	fn parse(parser: &mut Parser<'_, 'e>) -> Result<Option<Self::Output>> {
		if parser.advance_if('(').is_some() {
			parser.parse_grouped_expression().map(Some)
		} else if parser.advance_if(')').is_some() {
			Err(parser.error(ErrorKind::UnmatchedRightParen))
		} else {
			Ok(None)
		}
	}
}
