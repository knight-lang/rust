mod error;
mod opcode;
mod vm;

#[deprecated]
pub use crate::parser::{Parseable_OLD, Parser, SourceLocation};
pub use error::RuntimeError;
pub use opcode::Opcode;

#[deprecated]
pub use crate::program::Program;
pub use vm::*;

#[cfg(feature = "compliance")]
pub const MAX_VARIABLE_COUNT: usize = 65535;

#[deprecated]
pub use crate::parser::{ParseError, ParseErrorKind};
