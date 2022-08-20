#![allow(unused)]
// pub mod stream;

#[cfg(feature = "strict-numbers")]
/// The number type within Knight.
pub type Integer = i32;

#[cfg(not(feature = "strict-numbers"))]
/// The number type within Knight.
pub type Integer = i64;

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
pub use parser::{ParseError, Parser};
pub use value::Value;
