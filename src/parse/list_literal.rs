use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListLiteral<'e, I, E>(std::marker::PhantomData<(I, E, &'e ())>);

impl<'e, I: IntType, E: crate::value::text::Encoding> Parsable<'e, I, E> for ListLiteral<'e, I, E> {
	type Output = Value<'e, I, E>;

	fn parse(parser: &mut Parser<'_, 'e, I, E>) -> Result<Option<Self::Output>> {
		if !parser.env().flags().exts.list_literal || parser.advance_if('{').is_none() {
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
