mod opcode;
pub mod program;
mod vm;

pub use opcode::Opcode;
pub use program::{Builder, Program};
pub use vm::*;

cfg_if! {
	if #[cfg(feature = "compliance")] {
		pub const MAX_VARIABLE_LEN: usize = 127;
		pub const MAX_VARIABLE_COUNT: usize = 65535;
	}
}

#[derive(Error, Debug)]
pub enum ParseError {
	#[cfg(feature = "compliance")]
	#[error("variable name too long ({len} > {max}): {0:?}", len=.0.len(), max = MAX_VARIABLE_LEN)]
	VariableNameTooLong(crate::value::KString),

	#[cfg(feature = "compliance")]
	#[error("too many variables encountered (only {MAX_VARIABLE_LEN} allowed)")]
	TooManyVariables,
}
