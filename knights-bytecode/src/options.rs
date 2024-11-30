use crate::strings::Encoding;

#[derive(Default, Clone)]
pub struct Options {
	pub encoding: Encoding,

	#[cfg(feature = "compliance")]
	pub compliance: Compliance,

	#[cfg(feature = "extensions")]
	pub extensions: Extensions,
}

#[derive(Default, Clone)]
#[cfg(feature = "compliance")]
pub struct Compliance {
	pub check_container_length: bool, // make sure containers are within `i32::MAX`
	pub i32_integer: bool,
	pub check_overflow: bool,
	pub check_integer_function_bounds: bool,
	pub variable_name_length: bool,
	pub variable_count: bool,
	pub forbid_trailing_tokens: bool,
}

cfg_if! {
if #[cfg(feature = "extensions")] {
	#[derive(Default, Clone)]
	pub struct Extensions {
		pub builtin_fns: BuiltinFns,
		pub syntax: Syntax,
		pub types: Types,
		pub breaking: BreakingChanges,
	}

	#[derive(Default, Clone)]
	pub struct Types {
		pub floats: bool, // not working, potential future idea.
		pub hashmaps: bool, // not working, potential future idea.
		pub classes: bool, // not working, potential future idea.
	}

	#[derive(Default, Clone)]
	pub struct BreakingChanges {
		pub negate_reverses_collections: bool, // not working, potential future idea.
	}

	#[derive(Default, Clone)]
	pub struct Syntax {
		pub list_literals: bool, // not working
		pub string_interpolation: bool, // not working
		pub control_flow: bool, // XBREAK, XCONTINUE, XRETURN : partially working
	}

	#[derive(Default, Clone)]
	pub struct BuiltinFns {
		pub boolean: bool,
		pub string: bool,
		pub list: bool,
		pub integer: bool,
		pub null: bool,
	}
}}
