mod error;
mod opcode;
mod vm;

#[cfg(feature = "stacktrace")]
mod stacktrace;
#[cfg(feature = "stacktrace")]
pub use stacktrace::Stacktrace;

pub use error::RuntimeError;
pub use opcode::Opcode;

#[deprecated]
pub use crate::program::Program;
pub use vm::*;

#[cfg(feature = "compliance")]
pub const MAX_VARIABLE_COUNT: usize = 65535;

#[deprecated]
pub use crate::parser::{ParseError, ParseErrorKind};
