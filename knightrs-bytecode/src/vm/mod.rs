mod error;
pub mod opcode;
#[cfg(feature = "stacktrace")]
mod stacktrace;
mod vm;

pub use error::RuntimeError;
pub use opcode::Opcode;
#[cfg(feature = "stacktrace")]
pub use stacktrace::{Callsite, Stacktrace};
pub use vm::*;

#[cfg(feature = "compliance")]
// pub const MAX_VARIABLE_COUNT: usize = 65535;
pub const MAX_VARIABLE_COUNT: usize = 10;
