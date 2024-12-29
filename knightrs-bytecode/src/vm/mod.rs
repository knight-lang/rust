mod callsite;
mod error;
pub mod opcode;
mod vm;

#[cfg(feature = "stacktrace")]
mod stacktrace;
#[cfg(feature = "stacktrace")]
pub use stacktrace::Stacktrace;

pub use callsite::Callsite;
pub use error::RuntimeError;
pub use opcode::Opcode;
pub use vm::*;

#[cfg(feature = "compliance")]
// pub const MAX_VARIABLE_COUNT: usize = 65535;
pub const MAX_VARIABLE_COUNT: usize = 10;
