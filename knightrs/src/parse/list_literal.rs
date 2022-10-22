use super::*;

/// A [`Parsable`] for the `{ ... }` list literal syntax.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListLiteral<I, E>(std::marker::PhantomData<(I, E)>);

impl<I: IntType, E> Parsable<I, E> for ListLiteral<I, E> {
	type Output = Value<I, E>;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> Result<Option<Self::Output>> {
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
