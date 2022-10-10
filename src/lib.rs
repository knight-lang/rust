#![allow(clippy::module_inception)]
#![feature(trace_macros)]
#![feature(let_else)]
#![cfg_attr(docsrs, feature(doc_cfg))]
extern crate static_assertions as sa;

#[macro_use]
extern crate cfg_if;

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
