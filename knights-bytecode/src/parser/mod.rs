mod parser;
pub use parser::*;

#[cfg(feature = "compliance")]
pub const MAX_VARIABLE_LEN: usize = 127;
