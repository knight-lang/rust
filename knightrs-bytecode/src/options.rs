use crate::strings::Encoding;

#[derive(Default, Clone)]
pub struct Options {
	pub encoding: Encoding,

	#[cfg(feature = "compliance")]
	pub compliance: Compliance,

	#[cfg(feature = "extensions")]
	pub extensions: Extensions,

	#[cfg(feature = "qol")]
	pub qol: QualityOfLife,

	#[cfg(feature = "embedded")]
	pub embedded: Embedded,
}

#[derive(Default, Clone)]
#[cfg(feature = "qol")]
pub struct QualityOfLife {
	pub stacktrace: bool,
	pub check_parens: bool, // TODO: also make this strict compliance
}

#[derive(Default, Clone)]
#[cfg(feature = "embedded")]
pub struct Embedded {
	pub dont_exit_when_quitting: bool,
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
	pub check_equals_params: bool,
	pub limit_rand_range: bool,
	pub cant_dump_blocks: bool,
	pub check_quit_status_codes: bool,
	pub disallow_negative_int_to_list: bool,
	pub disable_all_extensions: bool, // TODO
	pub no_block_conversions: bool,
}

cfg_if! {
if #[cfg(feature = "extensions")] {
	#[derive(Default, Clone)]
	pub struct Extensions {
		pub builtin_fns: BuiltinFns,
		pub syntax: Syntax,
		pub types: Types,
		pub breaking: BreakingChanges,
		pub functions: Functions,
		pub negative_indexing: bool,
		pub argv: bool,
	}

	#[derive(Default, Clone)]
	pub struct Types {
		pub floats: bool, // not working, potential future idea.
		pub hashmaps: bool, // not working, potential future idea.
		pub classes: bool, // not working, potential future idea.
	}

	#[derive(Default, Clone)]
	pub struct Functions {
		pub eval: bool,
		pub value: bool,
	}

	#[derive(Default, Clone)]
	pub struct BreakingChanges {
		pub negate_reverses_collections: bool, // not working, potential future idea.
		pub random_can_be_negative: bool,
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

		pub length_of_anything: bool,
		pub assign_to_strings: bool,
		pub assign_to_random: bool,
	}
}}
