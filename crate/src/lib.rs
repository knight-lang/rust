// #![warn(missing_docs, missing_doc_code_examples)]
#![allow(clippy::tabs_in_doc_comments, unused)]
#![warn(/*, missing_doc_code_examples, missing_docs*/)]

pub mod function;
pub mod text;
mod value;
mod error;
mod stream;
pub mod environment;

/// The number type within Knight.
pub type Number = i64;

/// The boolean type within Knight.
pub type Boolean = bool;

#[doc(inline)]
pub use text::Text;

#[doc(inline)]
pub use function::Function;

pub use stream::{Stream, ParseError};
pub use environment::{Environment, Variable};
pub use value::Value;
pub use error::Error;
