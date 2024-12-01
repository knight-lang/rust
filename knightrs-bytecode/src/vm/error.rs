use crate::value::KString;
use crate::vm::SourceLocation;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct RuntimeError {
	err: crate::Error,

	#[cfg(feature = "stacktrace")]
	stacktrace: Vec<(Option<KString>, SourceLocation)>,
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "runtime error: {}", self.err)?;

		#[cfg(feature = "stacktrace")]
		for arg in &self.stacktrace {
			if let Some(name) = arg.0.as_deref() {
				write!(f, "\n\tin {} (function {})", arg.1, name)?;
			} else {
				write!(f, "\n\tin {}", arg.1)?;
			}
		}

		Ok(())
	}
}
