#![allow(clippy::module_inception)]
#![feature(let_else, int_log)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(debug_assertions, allow(deprecated))]

#[macro_use]
extern crate cfg_if;

mod ast;
mod containers;
pub mod env;
mod error;
mod function;
pub mod parse;
pub mod value;

pub use ast::Ast;
pub use containers::{Mutable, RefCount};
pub use error::{Error, Result};
pub use function::Function;
