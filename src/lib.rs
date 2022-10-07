#![feature(let_else)]
#![allow(clippy::module_inception)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

extern crate static_assertions as sa;

pub mod ast;
mod containers;
pub mod env;
mod error;
mod function;
pub mod parse;
pub mod value;

pub use ast::Ast;
pub use containers::{Mutable, RefCount};
pub use env::{Environment, Variable};
pub use error::{Error, Result};
pub use function::Function;
pub use parse::{Error as ParseError, Parser};
pub use value::*;

pub fn play(input: &str) -> Result<Value<'_>> {
	Environment::default().play(input.try_into()?)
}
