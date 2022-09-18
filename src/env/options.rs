#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Options {
	pub forbid_trailing_tokens: bool,
	pub dont_check_parens: bool,

	pub validate_variable_length: bool,
	pub validate_variable_contents: bool, // only for use with `VALUE`

	pub assign_to_prompt: bool,
	pub assign_to_system: bool,
	pub assign_to_string: bool,
	pub assign_to_list: bool,

	pub strict_call_argument: bool,
	pub check_quit_argument: bool,

	pub list_extensions: bool,
	pub string_extensions: bool,

	pub strict_equality: bool,

	pub negative_indexing: bool,
}
