#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Options {
	pub forbid_trailing_tokens: bool,
	pub dont_check_parens: bool,

	pub validate_variable_length: bool,
	pub validate_variable_contents: bool, // only for use with `VALUE`
}
