// #![warn(missing_docs, missing_doc_code_examples)]
#![allow(clippy::tabs_in_doc_comments, unused)]
#![warn(/*, missing_doc_code_examples, missing_docs*/)]

#[macro_use]
extern crate cfg_if;

extern crate static_assertions as sa;

macro_rules! debug_assert_const {
	($cond:expr) => { #[cfg(debug_assertions)] { let _ = [()][!$cond as usize]; }};
}

macro_rules! debug_assert_eq_const {
	($lhs:expr, $rhs:expr) => (debug_assert_const!($lhs == $rhs));
}

macro_rules! debug_assert_ne_const {
	($lhs:expr, $rhs:expr) => (debug_assert_const!($lhs != $rhs));
}

pub mod function;
pub mod text;
mod value;
mod ast;
mod error;
mod stream;
pub mod environment;
pub mod value2;
pub mod number;

cfg_if! {
	if #[cfg(all(feature="strict-numbers", feature="large-numbers"))] {
		compile_error!("cannot enable both strict-numbers and large-numbers");
	} else if #[cfg(feature="strict-numbers")] {
		/// The number type within Knight.
		pub type Number = i32;
	} else if #[cfg(feature = "large-numbers")] {
		/// The number type within Knight.
		pub type Number = i128;
	} else {
		/// The number type within Knight.
		pub type Number = i64;
	}
}

/// The boolean type within Knight.
pub type Boolean = bool;

#[doc(inline)]
pub use text::Text;

#[doc(inline)]
pub use function::Function;

pub use ast::Ast;

pub use stream::{Stream, ParseError};
pub use environment::{Environment, Variable};
pub use value::Value;
pub use error::{Error, Result};
