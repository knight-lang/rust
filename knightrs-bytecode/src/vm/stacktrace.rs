use super::Callsite;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Stacktrace<'src, 'path>(Vec<Callsite<'src, 'path>>);

impl<'src, 'path> Stacktrace<'src, 'path> {
	pub fn new(iter: impl IntoIterator<Item = Callsite<'src, 'path>>) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl Display for Stacktrace<'_, '_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		for callsite in &self.0 {
			write!(f, "\n\tin {callsite}")?;
		}

		Ok(())
	}
}
