use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blank;

pub enum Never {}

impl From<Never> for Value<'_> {
	fn from(never: Never) -> Self {
		match never {}
	}
}

impl<'e> Parsable<'e> for Blank {
	type Output = Never;

	fn parse(parser: &mut Parser<'_, 'e>) -> Result<Option<Self::Output>> {
		if parser.strip_whitespace_and_comments() {
			Err(parser.error(ErrorKind::RestartParsing))
		} else {
			Ok(None)
		}
	}
}
