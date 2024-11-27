#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(debug_assertions, allow(deprecated))]

#[macro_use]
extern crate cfg_if;

mod ast;
mod containers;
pub mod env;
mod error;
pub mod function;
pub mod parse;
pub mod value;

pub use ast::Ast;
pub use error::{Error, Result};
