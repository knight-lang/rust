use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blank;

pub enum Never {}

impl<I: super::IntType> From<Never> for Value<'_, I> {
	fn from(never: Never) -> Self {
		match never {}
	}
}

impl<'e, I: IntType> Parsable<'e, I> for Blank {
	type Output = Never;

	fn parse(parser: &mut Parser<'_, 'e, I>) -> Result<Option<Self::Output>> {
		if parser.strip_whitespace_and_comments() {
			Err(parser.error(ErrorKind::RestartParsing))
		} else {
			Ok(None)
		}
	}
}
