use crate::parser::SourceLocation;
use crate::parser::VariableName;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Stacktrace(Vec<(Option<VariableName>, SourceLocation)>);

impl Stacktrace {
	pub fn new(iter: impl IntoIterator<Item = (Option<VariableName>, SourceLocation)>) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl Display for Stacktrace {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		for arg in &self.0 {
			if let Some(ref name) = arg.0 {
				write!(f, "\n\tin {} (function {})", arg.1, name)?;
			} else {
				write!(f, "\n\tin {}", arg.1)?;
			}
		}

		Ok(())
	}
}
