#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Options {
	pub compliance: StrictCompliance,
	pub spec_extensions: SpecDefined,
	pub compiler: CompilerExtensions,
}

// 	pub forbid_trailing_tokens: bool,
// 	pub dont_check_parens: bool,

// 	pub validate_variable_length: bool,
// 	pub validate_variable_contents: bool, // only for use with `VALUE`

// 	pub strict_call_argument: bool,
// 	pub check_quit_argument: bool,

// 	pub list_extensions: bool,
// 	pub string_extensions: bool,

// 	pub strict_equality: bool,
// 	pub negative_indexing: bool,
// }

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CompilerExtensions {
	pub negative_indexing: bool,
	pub assign_to_prompt: bool,
	pub assign_to_system: bool,
	pub list_extensions: bool,
	pub string_extensions: bool,
	pub srand_fn: bool,
	pub range_fn: bool,
	pub reverse_fn: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SpecDefined {
	// functions
	pub value_fn: bool,
	pub assign_to_string: bool,
	pub assign_to_list: bool,
	pub handle_fn: bool,
	pub yeet_fn: bool,
	pub use_fn: bool,
	pub system_fn: bool,
	pub eval_fn: bool,

	// sugar
	pub interpolation: bool,
	pub list_literal: bool,

	// types
	pub floats: bool,
	pub maps: bool,
	pub objects: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StrictCompliance {
	pub single_expression: bool,
	pub container_length: bool,
	pub knight_encoding: bool,
	pub checked_overflow: bool,
	pub i32_integer: bool,
	pub call_argument: bool,
	pub variable_name: bool,
	pub check_quit_argument: bool,
	pub strict_equality: bool,
	pub restrict_rand: bool,
}
