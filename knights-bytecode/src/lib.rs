#![allow(unused)]
#![warn(unsafe_op_in_unsafe_fn)]

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate thiserror;

mod container;
pub mod env;
pub mod error;
pub mod old_vm_and_parser_and_program;
pub mod options;
pub mod strings;
pub mod value;
pub use env::Environment;
pub use error::{Error, Result};
pub use value::Value;
