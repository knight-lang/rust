use super::*;

/// A [`Parsable`] that strips whitespace and comments.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blank;

/// The never type's replacement.
pub enum Never {}

impl From<Never> for Value {
	fn from(never: Never) -> Self {
		match never {}
	}
}

impl Parsable for Blank {
	type Output = Never;

	fn parse(parser: &mut Parser<'_, '_>) -> Result<Option<Self::Output>> {
		if parser.strip_whitespace_and_comments().is_some() {
			Err(parser.error(ErrorKind::RestartParsing))
		} else {
			Ok(None)
		}
	}
}
