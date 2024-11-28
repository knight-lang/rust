use crate::strings::Encoding;

#[derive(Default)]
pub struct Options {
	#[cfg(feature = "compliance")]
	pub compliance: Compliance,

	pub encoding: Encoding,
}

#[derive(Default)]
#[cfg(feature = "compliance")]
pub struct Compliance {
	pub check_length: bool,
	pub i32_integer: bool,
	pub check_overflow: bool,
	pub check_integer_function_bounds: bool,
}
