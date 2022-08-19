#![allow(unused)]
// pub mod stream;

pub mod ast;
pub mod env;
mod error;
mod function;
mod knightstr;
pub mod parser;
pub mod text;
pub mod value;

pub use ast::Ast;
pub use env::Environment;
pub use error::{Error, Result};
pub use function::Function;
pub use knightstr::KnightStr;
pub use text::Text;
pub use value::{Number, Value};
