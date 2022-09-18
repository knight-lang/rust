#![feature(let_else)]
#![allow(clippy::module_inception)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

extern crate static_assertions as sa;

#[macro_use]
extern crate cfg_if;

pub mod ast;
mod containers;
pub mod env;
mod error;
mod function;
pub mod parser;
pub mod value;
mod variable;

pub use ast::Ast;
pub use containers::{Mutable, RefCount};
pub use env::Environment;
pub use error::{Error, Result};
pub use function::Function;
pub use parser::{Error as ParseError, Parser};
pub use value::*;
pub use variable::Variable;
