use crate::strings::Encoding;

#[derive(Default)]
pub struct Options {
	pub encoding: Encoding,

	#[cfg(feature = "compliance")]
	pub compliance: Compliance,

	#[cfg(feature = "extensions")]
	pub extensions: Extensions,
}

#[derive(Default)]
#[cfg(feature = "compliance")]
pub struct Compliance {
	pub check_length: bool,
	pub i32_integer: bool,
	pub check_overflow: bool,
	pub check_integer_function_bounds: bool,
}

#[derive(Default)]
#[cfg(feature = "extensions")]
pub struct Extensions {
	pub types: Types,
}

#[derive(Default)]
#[cfg(feature = "extensions")]
pub struct Types {
	pub boolean: bool,
	pub string: bool,
	pub list: bool,
	pub integer: bool,
	pub null: bool,
}
