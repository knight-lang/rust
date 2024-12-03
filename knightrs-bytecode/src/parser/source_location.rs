use crate::container::RefCount;
use crate::parser::{ParseError, ParseErrorKind};
use std::fmt::{self, Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SourceLocation {
	// TODO: don't refcount this. maybe have all parser errors be lifetime-bounded? And then have
	// the `eval` somehow leak them or something?
	filename: Option<RefCount<Path>>,
	lineno: usize,
}

impl SourceLocation {
	pub fn new(filename: Option<RefCount<Path>>, lineno: usize) -> Self {
		Self { filename, lineno }
	}

	pub fn error(self, kind: ParseErrorKind) -> ParseError<'static> {
		ParseError { whence: self, kind, _ignored: &() }
	}
}

impl Display for SourceLocation {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if let Some(ref filename) = self.filename {
			write!(f, "{}:{}", filename.display(), self.lineno)
		} else {
			write!(f, "<expr>:{}", self.lineno)
		}
	}
}
