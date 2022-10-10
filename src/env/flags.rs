#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flags {
	pub assign_to: AssignToFlags,

	#[cfg(feature = "compliance")]
	pub compliance: ComplianceFlags,

	pub negative_indexing: bool,

	#[cfg(feature = "extensions")]
	pub exts: ExtensionFlags,

	pub fns: FunctionFlags,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "compliance")]
pub struct ComplianceFlags {
	pub check_quit_bounds: bool,
	pub forbid_trailing_tokens: bool,
	pub verify_variable_names: bool,
	pub check_call_arg: bool,
	pub limit_rand_range: bool,
	pub check_equals_params: bool,
	pub check_integer_function_bounds: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionFlags {
	pub value: bool,
	pub eval: bool,
	pub handle: bool,
	pub yeet: bool,
	pub r#use: bool,
	pub system: bool,
	pub xsrand: bool,
	pub xreverse: bool,
	pub xrange: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "extensions")]
pub struct ExtensionFlags {
	pub ascii_on_lists: bool,
	pub boolean: bool,
	pub list: bool,
	pub text: bool,
	pub list_literal: bool,
	pub negative_ranges: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssignToFlags {
	pub prompt: bool,
	pub system: bool,
	pub list: bool,
	pub text: bool,
}
