#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flags {
	pub assign_to: AssignToFlags,
	pub compliance: ComplianceFlags,
	pub negative_indexing: bool,
	pub exts: ExtensionFlags,
	pub fns: FunctionFlags,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComplianceFlags {
	pub check_quit_bounds: bool,
	pub forbid_trailing_tokens: bool,
	pub verify_variable_names: bool,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtensionFlags {
	pub ascii_on_lists: bool,
	pub boolean: bool,
	pub list: bool,
	pub text: bool,
	pub list_literal: bool,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssignToFlags {
	pub prompt: bool,
	pub system: bool,
	pub list: bool,
	pub text: bool,
}
