pub mod ast;
pub mod env;
pub mod func;
#[macro_use]
pub mod text;
pub mod value;
mod parse;
mod shared;

use shared::{malloc, free};
pub use value::{Value, Number};
pub use ast::Ast;
pub use text::Text;
pub use env::Variable;

pub unsafe fn startup() {
	func::startup();
	env::startup();
}

pub unsafe fn shutdown() {
	env::shutdown();
	ast::cleanup();
	text::cleanup();
}

pub unsafe fn run(input: *const u8) -> Value {
	let parsed = parse::parse(input);

	#[cfg(not(feature="reckless"))]
	if parsed == value::UNDEFINED {
		panic!("unable to parse stream");
	}

	let ret = value::run(parsed);
	value::free(parsed);
	ret
}
