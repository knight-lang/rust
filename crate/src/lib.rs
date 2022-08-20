#![allow(unused)]

extern crate static_assertions as sa;
#[macro_use]
extern crate cfg_if;

cfg_if! {
	if #[cfg(feature = "strict-numbers")] {
		/// The number type within Knight.
		pub type Integer = i32;
	} else {
		/// The number type within Knight.
		pub type Integer = i64;
	}
}

cfg_if! {
	if #[cfg(feature = "multithreaded")] {
		type RefCount<T> = std::sync::Arc<T>;
		type Mutable<T> = std::sync::RwLock<T>;
	} else {
		type RefCount<T> = std::rc::Rc<T>;
		type Mutable<T> = std::cell::RefCell<T>;
	}
}

cfg_if! {
	if #[cfg(not(feature = "no-arrays"))] {
		mod array;
		pub use array::Array;
	}
}

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
