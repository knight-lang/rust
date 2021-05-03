#![allow(unused)]
extern crate static_assertions as sa;

macro_rules! debug_assert_const {
	($cond:expr) => (debug_assert_const!($cond, concat!("assertion failed: ", stringify!($cond))));
	($cond:expr, $msg:expr) => (#[cfg(debug_assertions)] {let _ = [$msg][!$cond as usize]; })
}

macro_rules! debug_assert_eq_const {
	($lhs:expr, $rhs:expr) => (debug_assert_const!($lhs == $rhs));
	($lhs:expr, $rhs:expr, $msg:expr) => (debug_assert_const!($lhs == $rhs, $msg))
}

macro_rules! debug_assert_ne_const {
	($lhs:expr, $rhs:expr) => (debug_assert_const!($lhs != $rhs));
	($lhs:expr, $rhs:expr, $msg:expr) => (debug_assert_const!($lhs != $rhs, $msg))
}

pub mod value;
mod text;
mod number;
mod env;
mod ast;
mod null;
mod boolean;
mod error;
pub mod functions;

pub use error::*;

pub use text::*;
pub use number::*;
pub use boolean::*;
pub use env::*;
pub use ast::*;
pub use value::*;
pub use null::*;
pub use env::*;
