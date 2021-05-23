// #![warn(missing_docs, missing_doc_code_examples)]
#![allow(clippy::tabs_in_doc_comments, unused)]
#![warn(/*, missing_doc_code_examples, missing_docs*/)]

#[macro_use]
extern crate cfg_if;


cfg_if! {
	if #[cfg(feature="abort-on-errors")] {
		macro_rules! handle_error {
			($err:expr) => (panic!("runtime error: {}", $err))
		}
	} else if #[cfg(feature="unsafe-reckless")] {
		macro_rules! handle_error {
			($err:expr) => ({
				#[cfg(debug_assertions)] {
					unreachable!("reckless condition failed!")
				}

				#[cfg(not(debug_assertions))] unsafe {
					std::hint::unreachable_unchecked()
				}
			})
		}
	} else {
		macro_rules! handle_error {
			($err:expr) => (return Err($err))
		}
	}
}

pub mod function;
pub mod text;
mod value;
mod error;
mod stream;
mod ast;
pub mod environment;

cfg_if! {
	if #[cfg(feature="strict-numbers")] {
		/// The number type within Knight.
		pub type Number = i32;
	} else {
		/// The number type within Knight.
		pub type Number = i64;
	}
}

/// The boolean type within Knight.
pub type Boolean = bool;

#[doc(inline)]
pub use text::Text;

#[doc(inline)]
pub use function::Function;

pub use stream::{Stream, ParseError};
pub use environment::{Environment, Variable};
pub use value::Value;
pub use error::{Error, Result};
pub use ast::Ast;
