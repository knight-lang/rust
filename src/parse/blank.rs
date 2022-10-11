use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blank;

pub enum Never {}

impl<I: super::IntType, E> From<Never> for Value<'_, I, E> {
	fn from(never: Never) -> Self {
		match never {}
	}
}

impl<'e, I: IntType, E: crate::value::text::Encoding> Parsable<'e, I, E> for Blank {
	type Output = Never;

	fn parse(parser: &mut Parser<'_, 'e, I, E>) -> Result<Option<Self::Output>> {
		if parser.strip_whitespace_and_comments() {
			Err(parser.error(ErrorKind::RestartParsing))
		} else {
			Ok(None)
		}
	}
}
