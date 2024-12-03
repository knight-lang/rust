use crate::parser::SourceLocation;
use crate::value::KString;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct RuntimeError<'path> {
	pub(super) err: crate::Error,

	#[cfg(feature = "stacktrace")]
	pub(super) stacktrace: super::Stacktrace<'path>,
}

impl Display for RuntimeError<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "runtime error: {}", self.err)?;

		#[cfg(feature = "stacktrace")]
		write!(f, "{}", self.stacktrace)?;

		Ok(())
	}
}
