#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate cfg_if;


// todo: use `NonNull`.

macro_rules! likely {
	($cond:expr) => (unlikely!($cond))
}

macro_rules! unlikely {
	($cond:expr) => ({
		#[cold]
		fn unlikely(){}

		let cond = $cond;
		if cond { unlikely() }
		cond
	})
}

macro_rules! debug_assert_const {
	($value:expr) => ({
		#[cfg(debug_assertions)] let _: () = [()][!($value as bool) as usize];
	});
}

macro_rules! debug_assert_eq_const {
	($lhs:expr, $rhs:expr) => ({
		#[cfg(debug_assertions)] let _: () = [()][($lhs != $rhs) as bool as usize];
	});
}

// macro_rules! debug_assert_ne_const {
// 	($lhs:expr, $rhs:expr) => ({
// 		#[cfg(debug_assertions)] let _: () = [()][($lhs == $rhs) as bool as usize];
// 	});
// }

pub mod number;
pub mod value;
pub mod text;
pub mod ast;
pub mod variable;
mod boolean;
pub mod function;
mod custom;
mod null;
pub mod env;
mod error;
pub mod ops;

pub use null::Null;
pub use value::Value;
pub use boolean::Boolean;
pub use custom::Custom;
pub use text::Text;
pub use env::Environment;
pub use variable::Variable;
pub use ast::Ast;

pub use number::Number;
pub use error::{Error, Result};
pub use function::Function;
