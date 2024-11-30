mod opcode;
pub mod parser;
pub mod program;
mod vm;

pub use opcode::Opcode;
pub use parser::{Parseable, Parser, SourceLocation};
pub use program::{Builder, Program};
pub use vm::*;

cfg_if! {
	if #[cfg(feature = "compliance")] {
		pub const MAX_VARIABLE_LEN: usize = 127;
		pub const MAX_VARIABLE_COUNT: usize = 65535;
	}
}

#[derive(Debug)]
pub struct ParseError {
	pub whence: (Option<std::path::PathBuf>, usize),
	pub kind: ParseErrorKind,
}

use std::fmt::{self, Display, Formatter};

use crate::strings::Encoding;
impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if let Some(ref pathbuf) = self.whence.0 {
			write!(f, "{}.{}: {}", pathbuf.display(), self.whence.1, self.kind)
		} else {
			write!(f, "<expr>.{}: {}", self.whence.1, self.kind)
		}
	}
}
impl std::error::Error for ParseError {}

#[derive(Error, Debug)]
pub enum ParseErrorKind {
	#[cfg(feature = "compliance")]
	#[error("variable name too long ({len} > {max}): {0:?}", len=.0.len(), max = MAX_VARIABLE_LEN)]
	VariableNameTooLong(crate::value::KString),

	#[cfg(feature = "compliance")]
	#[error("too many variables encountered (only {MAX_VARIABLE_LEN} allowed)")]
	TooManyVariables,

	#[cfg(feature = "compliance")]
	#[error("invalid character {1:?} for encoding {0:?}")]
	InvalidCharInEncoding(Encoding, char),

	#[cfg(feature = "compliance")]
	#[error("there were additional tokens in the source")]
	TrailingTokens,

	// There was nothing to parse
	#[error("there was nothing to parse.")]
	EmptySource,

	#[error("character doesn't start a token: {0:?}")]
	UnknownTokenStart(char),

	#[error("integer literal overflowed")]
	IntegerLiteralOverflow,
}

impl ParseErrorKind {
	// this tuple is a huge hack. maybe when i remove it i can also remove `'filename`
	pub fn error(self, whence: ((Option<std::path::PathBuf>, usize))) -> ParseError {
		ParseError { whence, kind: self }
	}
}
