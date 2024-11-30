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
	pub variable_name_length: bool,
	pub variable_count: bool,
	pub forbid_trailing_tokens: bool,
}

#[derive(Default)]
#[cfg(feature = "extensions")]
pub struct Extensions {
	pub builtin_fns: BuiltinFns,
	pub syntax: Syntax,
	pub floats: bool, // not working
}

#[derive(Default)]
#[cfg(feature = "extensions")]
pub struct Syntax {
	pub list_literals: bool,
	pub string_interpolation: bool, // not working
}

#[derive(Default)]
#[cfg(feature = "extensions")]
// TODO: rename from types (which imlpies new types) to "funciton extensions" or somethin
pub struct BuiltinFns {
	pub boolean: bool,
	pub string: bool,
	pub list: bool,
	pub integer: bool,
	pub null: bool,
}
