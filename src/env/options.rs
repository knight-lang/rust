#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Options {
	pub forbid_trailing_tokens: bool,
	pub dont_check_parens: bool,
}
