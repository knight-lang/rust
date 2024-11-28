#![allow(unused)]
#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate thiserror;

mod container;
pub mod env;
pub type KString = strings::String;
pub mod options;
// pub mod parser;
pub mod error;
pub mod strings;
pub mod value;
pub mod vm;
pub use error::{Error, Result};
