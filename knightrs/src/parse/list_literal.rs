use super::*;

/// A [`Parsable`] for the `{ ... }` list literal syntax.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListLiteral<I>(std::marker::PhantomData<I>);

impl<I: IntType> Parsable<I> for ListLiteral<I> {
	type Output = Value<I>;

	fn parse(parser: &mut Parser<'_, '_, I>) -> Result<Option<Self::Output>> {
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
