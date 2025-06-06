use crate::parser::{SourceLocation, VariableName};
use std::fmt::{self, Display, Formatter};

/// A stacktrace---the list of [`Callsite`]s a program's been to.
#[derive(Debug, Clone)]
pub struct Stacktrace<'src, 'path>(Box<[Callsite<'src, 'path>]>);

impl<'src, 'path> Stacktrace<'src, 'path> {
	/// Create a new [`Stacktrace`] from the list of callsites.
	pub fn new(iter: impl IntoIterator<Item = Callsite<'src, 'path>>) -> Self {
		Self(iter.into_iter().collect())
	}

	/// Get all the callsites in this stacktrace.
	pub fn callsites(&self) -> &[Callsite<'src, 'path>] {
		&self.0
	}
}

impl Display for Stacktrace<'_, '_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		for callsite in self.callsites() {
			write!(f, "\n\tin {callsite}")?;
		}

		Ok(())
	}
}

/// A location within a program's execution.
#[derive(Debug, Clone)]
pub struct Callsite<'src, 'path> {
	location: SourceLocation<'path>,
	fn_name: Option<VariableName<'src>>,
}

impl<'src, 'path> Callsite<'src, 'path> {
	/// Creates a new [`Callsite`]. The `fn_name` can be supplied to indicate the call happened within
	/// a function.
	pub fn new(fn_name: Option<VariableName<'src>>, location: SourceLocation<'path>) -> Self {
		Self { location, fn_name }
	}

	/// The name of function, if present.
	pub fn fn_name(&self) -> Option<&VariableName<'src>> {
		self.fn_name.as_ref()
	}

	/// The name of function, if present.
	pub fn location(&self) -> SourceLocation<'path> {
		self.location
	}
}

impl Display for Callsite<'_, '_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.location)?;

		if let Some(ref fn_name) = self.fn_name {
			write!(f, " (function {})", fn_name)?;
		}

		Ok(())
	}
}
