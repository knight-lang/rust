use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blank;

pub enum Never {}

impl<I, E> From<Never> for Value<I, E> {
	fn from(never: Never) -> Self {
		match never {}
	}
}

impl<I, E: Encoding> Parsable<I, E> for Blank {
	type Output = Never;

	fn parse(parser: &mut Parser<'_, '_, I, E>) -> Result<Option<Self::Output>> {
		if parser.strip_whitespace_and_comments() {
			Err(parser.error(ErrorKind::RestartParsing))
		} else {
			Ok(None)
		}
	}
}
