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

cfg_if! {
if #[cfg(feature = "extensions")] {
	#[derive(Default)]
	pub struct Extensions {
		pub builtin_fns: BuiltinFns,
		pub syntax: Syntax,
		pub types: Types,
	}

	#[derive(Default)]
	pub struct Types {
		pub floats: bool, // not working, potential future idea.
		pub hashmaps: bool, // not working, potential future idea.
		pub classes: bool, // not working, potential future idea.
	}

	#[derive(Default)]
	pub struct Syntax {
		pub list_literals: bool, // not working
		pub string_interpolation: bool, // not working
	}

	#[derive(Default)]
	pub struct BuiltinFns {
		pub boolean: bool,
		pub string: bool,
		pub list: bool,
		pub integer: bool,
		pub null: bool,
	}
}}
