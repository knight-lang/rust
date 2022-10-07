#![feature(let_else)]
#![allow(clippy::module_inception)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub mod ast;
pub mod env;
mod error;
mod function;
pub mod parser;
pub mod value;
mod variable;

pub use ast::Ast;
pub use env::Environment;
pub use error::{Error, Result};
pub use function::Function;
pub use parser::{Error as ParseError, Parser};
pub use value::*;
pub use variable::Variable;
