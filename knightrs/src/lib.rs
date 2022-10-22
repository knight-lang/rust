#![allow(clippy::module_inception)]
#![feature(let_else, int_log)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(debug_assertions, allow(deprecated))]

#[macro_use]
extern crate cfg_if;

pub mod ast;
mod containers;
pub mod env;
mod error;
mod function;
pub mod parse;
pub mod value;

pub use ast::Ast;
pub use containers::{Mutable, RefCount};
pub use env::{Environment, Variable};
pub use error::{Error, Result};
pub use function::Function;
pub use parse::{Error as ParseError, Parser};
pub use value::*;

#[cfg(feature = "play")]
#[cfg_attr(docsrs, doc(cfg(feature = "play")))]
pub fn play(
	encoding: &str,
	inttype: &str,
	overflow: &str,
	src: &str,
	flags: &env::Flags,
) -> Result<()> {
	macro_rules! play {
		(E; "ascii") => (crate::value::text::Ascii);
		(E; "knight-encoding") => (crate::value::text::KnightEncoding);
		(E; "utf8") => (crate::value::text::Utf8);
		(I; "i32") => (i32);
		(I; "i64") => (i64);
		(C; "checked" $x:tt) => (crate::value::integer::Checked<play![I; $x]>);
		(C; "wrapping" $x:tt) => (play![I; $x]);
		($($e:tt $i:tt $c:tt),* $(,)?) => {
			match (encoding, inttype, overflow) {
				$(($e, $i, $c) => {
					let mut env = Environment::<'_, play![C; $c $i], play![E; $e]>::builder(flags).build();
					env.play(TextSlice::new(src, flags)?).and(Ok(()))
				})*
				_ => panic!("bad options: encoding: {encoding:?}, inttype: {inttype:?}, overflow: {overflow:?}")
			}
		};
	}

	play! {
		"knight-encoding" "i32" "checked",
		"knight-encoding" "i32" "wrapping",
		"knight-encoding" "i64" "checked",
		"knight-encoding" "i64" "wrapping",

		"ascii" "i32" "checked",
		"ascii" "i32" "wrapping",
		"ascii" "i64" "checked",
		"ascii" "i64" "wrapping",

		"utf8" "i32" "checked",
		"utf8" "i32" "wrapping",
		"utf8" "i64" "checked",
		"utf8" "i64" "wrapping",
	}
}
