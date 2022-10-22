use super::*;

/// A [`Parsable`] for the `{ ... }` list literal syntax.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListLiteral;

impl Parsable for ListLiteral {
	type Output = Value;

	fn parse(parser: &mut Parser<'_, '_>) -> Result<Option<Self::Output>> {
		if !parser.env().flags().extensions.list_literal || parser.advance_if('{').is_none() {
			return Ok(None);
		}

		let mut expansion = Value::List(Default::default());
		while {
			parser.strip_whitespace_and_comments();
			parser.advance_if('}').is_none()
		} {
			expansion = Value::Ast(crate::Ast::new(
				crate::function::ADD(),
				vec![expansion, parser.parse_expression()?].into(),
			));
		}

		Ok(Some(expansion))
	}
}
