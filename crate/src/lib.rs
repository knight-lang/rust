#![allow(unused)]
// pub mod stream;

pub mod ast;
pub mod env;
mod error;
mod function;
pub mod knstr;
pub mod parser;
pub mod value;

pub use crate::knstr::{KnStr, SharedStr};
pub use ast::Ast;
pub use env::Environment;
pub use error::{Error, Result};
pub use function::Function;
pub use value::{Number, Value};
