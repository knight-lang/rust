#![allow(unused)]
#![warn(unsafe_op_in_unsafe_fn)]

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate static_assertions as sa;

mod container;
pub mod env;
pub mod error;
pub mod options;
pub mod parser;
pub mod strings;
pub mod value;
pub mod vm;
pub use env::Environment;
pub use error::{Error, Result};
pub use value::Value;
