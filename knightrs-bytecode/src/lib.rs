#![cfg_attr(debug_assertions, allow(deprecated))] // allow our own deprecated stuff while debugging
#![warn(unsafe_op_in_unsafe_fn)]

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate thiserror;

extern crate static_assertions as sa;

macro_rules! unreachable_unchecked {
	($($body:tt)*) => {
		if cfg!(debug_assertions) {
			unreachable!($($body)*);
		} else {
			::std::hint::unreachable_unchecked();
		}
	}
}

#[allow(unused)] // TODO
macro_rules! cfg_expr {
	(feature = $feature:literal, $ift:expr, $iff:expr) => {{
		#[cfg(feature = $feature)]
		{
			$ift
		}
		#[cfg(not(feature = $feature))]
		{
			$iff
		}
	}};
}

mod container;
pub mod env;
pub mod error;
// #[warn(unused)]
pub mod gc;
pub mod options;
pub mod parser;
pub mod program;
pub mod strings;

pub mod value;

pub mod vm;
pub use env::Environment;
pub use error::{Error, Result};
pub use gc::Gc;
pub use options::Options;
pub use value::Value;
