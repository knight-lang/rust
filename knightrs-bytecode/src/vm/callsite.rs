use crate::parser::{SourceLocation, VariableName};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Callsite {
	src: SourceLocation,
	fn_name: Option<VariableName>,
}

impl Callsite {
	pub fn new(fn_name: Option<VariableName>, src: SourceLocation) -> Self {
		Self { src, fn_name }
	}
}

impl Display for Callsite {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.src)?;

		if let Some(ref fn_name) = self.fn_name {
			write!(f, " (function {})", fn_name)?;
		}

		Ok(())
	}
}
