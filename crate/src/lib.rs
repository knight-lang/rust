// #![warn(missing_docs, missing_doc_code_examples)]
#![allow(clippy::tabs_in_doc_comments, unused)]
#![warn(/*, missing_doc_code_examples, missing_docs*/)]

#![cfg_attr(not(feature="unsafe-enabled"), deny(unsafe_code))]

#[macro_use]
extern crate cfg_if;

#[cfg(all(feature="unsafe-reckless", feature="disallow-unicode"))]
compile_error!("'unsafe-reckless' may not be enabled with 'disallow-unicode'");

#[cfg(all(feature="unsafe-reckless", feature="checked-overflow"))]
compile_error!("'unsafe-reckless' may not be enabled with 'checked-overflow'");

cfg_if! {
	if #[cfg(feature="abort-on-errors")] {
		macro_rules! error_inplace {
			($err:expr) => (panic!("runtime error: {}", $err))
		}
	} else if #[cfg(feature="unsafe-reckless")] {
		macro_rules! error_inplace {
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
		macro_rules! error_inplace {
			($err:expr) => ($err)
		}
	}
}

macro_rules! error {
	($err:expr) => (Err(error_inplace!($err)));
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

#[doc(inline)]
pub use error::{Error, Result};
pub use ast::Ast;
