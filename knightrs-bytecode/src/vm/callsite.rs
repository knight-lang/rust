use crate::parser::{SourceLocation, VariableName};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Callsite<'src, 'path> {
	src: SourceLocation<'path>,
	fn_name: Option<VariableName<'src>>,
}

impl<'src, 'path> Callsite<'src, 'path> {
	pub fn new(fn_name: Option<VariableName<'src>>, src: SourceLocation<'path>) -> Self {
		Self { src, fn_name }
	}
}

impl Display for Callsite<'_, '_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.src)?;

		if let Some(ref fn_name) = self.fn_name {
			write!(f, " (function {})", fn_name)?;
		}

		Ok(())
	}
}
