// #![warn(missing_docs, missing_doc_code_examples)]
#![allow(clippy::tabs_in_doc_comments, unused)]
#![warn(/*, missing_doc_code_examples, missing_docs*/)]

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate static_assertions;

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
pub mod null;
mod boolean;
mod ast;
mod error;
mod stream;
pub mod environment;
pub mod value;
pub mod number;

pub use number::Number;
pub use null::Null;
pub use boolean::Boolean;
pub use text::Text;

#[doc(inline)]
pub use function::Function;

pub use ast::Ast;

pub use stream::{Stream, ParseError};
pub use environment::{Environment, Variable};
pub use value::Value;
pub use error::{Error, Result};
