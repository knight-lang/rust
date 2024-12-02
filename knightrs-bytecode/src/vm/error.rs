use crate::parser::SourceLocation;
use crate::value::KString;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct RuntimeError {
	pub(super) err: crate::Error,

	#[cfg(feature = "stacktrace")]
	pub(super) stacktrace: super::Stacktrace,
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "runtime error: {}", self.err)?;

		#[cfg(feature = "stacktrace")]
		write!(f, "{}", self.stacktrace)?;

		Ok(())
	}
}
