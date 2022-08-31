#[derive(Debug, Default)]
pub struct Functions {
	pub value: bool,
	pub eval: bool,
	pub handle: bool,
	pub r#use: bool,
	pub xsrand: bool,
	pub xreverse: bool,
}

#[derive(Debug, Default)]
pub struct Features {
	pub functions: Functions,
	pub assign_to_strings: bool,
}

impl Features {
	pub const fn new() -> Self {
		Self {
			functions: Functions {
				value: false,
				eval: false,
				handle: false,
				r#use: false,
				xsrand: false,
				xreverse: false,
			},
			assign_to_strings: false,
		}
	}
	pub fn populate_functions(&self, builder: &mut crate::env::Builder) {
		use crate::function::*;

		if self.functions.xsrand {
			builder.declare_function(&XSRAND);
		}

		if self.functions.xreverse {
			builder.declare_function(&XREVERSE);
		}

		if self.functions.value {
			builder.declare_function(&VALUE);
		}

		if self.functions.eval {
			builder.declare_function(&EVAL);
		}

		if self.functions.handle {
			builder.declare_function(&HANDLE);
		}

		if self.functions.r#use {
			builder.declare_function(&USE);
		}
	}
}
