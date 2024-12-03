use super::Callsite;
use crate::parser::SourceLocation;
use crate::parser::VariableName;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Stacktrace<'path>(Vec<Callsite<'path>>);

impl<'path> Stacktrace<'path> {
	pub fn new(iter: impl IntoIterator<Item = Callsite<'path>>) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl Display for Stacktrace<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		for callsite in &self.0 {
			write!(f, "\n\tin {callsite}")?;
		}

		Ok(())
	}
}
