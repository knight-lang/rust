#![cfg_attr(doc_cfg, feature(doc_cfg))]

extern crate static_assertions as sa;

#[macro_use]
extern crate cfg_if;

pub mod ast;
mod containers;
pub mod env;
mod error;
mod function;
mod list;
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
pub use list::List;
pub use parser::{ParseError, Parser};
pub use value::*;
pub use variable::Variable;
