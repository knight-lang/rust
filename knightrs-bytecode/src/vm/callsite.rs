use crate::parser::{SourceLocation, VariableName};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Callsite<'path> {
	src: SourceLocation<'path>,
	fn_name: Option<VariableName>,
}

impl<'path> Callsite<'path> {
	pub fn new(fn_name: Option<VariableName>, src: SourceLocation<'path>) -> Self {
		Self { src, fn_name }
	}
}

impl Display for Callsite<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.src)?;

		if let Some(ref fn_name) = self.fn_name {
			write!(f, " (function {})", fn_name)?;
		}

		Ok(())
	}
}
