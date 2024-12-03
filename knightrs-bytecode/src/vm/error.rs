use crate::parser::SourceLocation;
use crate::value::KString;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct RuntimeError<'src, 'path> {
	pub(super) err: crate::Error,

	#[cfg(feature = "stacktrace")]
	pub(super) stacktrace: super::Stacktrace<'src, 'path>,
}

impl Display for RuntimeError<'_, '_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "runtime error: {}", self.err)?;

		#[cfg(feature = "stacktrace")]
		write!(f, "{}", self.stacktrace)?;

		Ok(())
	}
}
