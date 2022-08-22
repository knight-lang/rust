#![allow(unused)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

extern crate static_assertions as sa;

#[macro_use]
extern crate cfg_if;

cfg_if! {
	if #[cfg(feature = "strict-numbers")] {
		/// The number type within Knight.
		pub type Integer = i32;
	} else {
		/// The number type within Knight.
		pub type Integer = i64;
	}
}

cfg_if! {
	if #[cfg(feature = "arrays")] {
		mod list;
		pub use list::List;
	}
}

pub mod ast;
mod containers;
pub mod env;
mod error;
mod function;
pub mod parser;
pub mod text;
pub mod value;
mod variable;

pub use crate::text::{SharedText, Text};
pub use ast::Ast;
pub use containers::{Mutable, RefCount};
pub use env::Environment;
pub use error::{Error, Result};
pub use function::Function;
pub use parser::{ParseError, Parser};
pub use value::Value;
pub use variable::Variable;
