#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate cfg_if;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;


// todo: use `NonNull`.

macro_rules! likely {
	($cond:expr) => (unlikely!($cond))
}

macro_rules! unlikely {
	() => (unlikely!(true));
	($cond:expr) => ({
		#[cold]
		const fn unlikely(){}

		let cond = $cond;
		if cond { unlikely() }
		cond
	})
}

macro_rules! debug_assert_const {
	($value:expr) => ({
		#[cfg(debug_assertions)] let _: () = [()][!($value as bool) as usize];
	});
}

macro_rules! debug_assert_eq_const {
	($lhs:expr, $rhs:expr) => ({
		#[cfg(debug_assertions)] let _: () = [()][($lhs != $rhs) as bool as usize];
	});
}

// macro_rules! debug_assert_ne_const {
// 	($lhs:expr, $rhs:expr) => ({
// 		#[cfg(debug_assertions)] let _: () = [()][($lhs == $rhs) as bool as usize];
// 	});
// }
/*
macro_rules! error {
	($($tt:tt)+) => {{
		#[cfg(feature="panic-on-error")]
		{ panic!("{}", $($tt)+) }

		#[cfg(not(feature="panic-on-error"))]
		{ Err($($tt)+) }
	}}
}
*/

macro_rules! if_feature {
	($feature:literal $if_true:block $(else $if_false:block)?) => {{
		#[cfg(feature=$feature)]
		$if_true

		$(#[cfg(not(feature=$feature))]
		$if_false)?
	}}
}

pub mod types;
pub use types::*;

pub mod value;
pub mod function;
pub mod env;
mod error;
pub mod ops;
pub mod parse;
pub use env::Environment;

pub use value::Value;
pub use error::{Error, Result};
pub use function::Function;

unsafe fn xalloc(layout: std::alloc::Layout) -> *mut u8 {
	let ptr = std::alloc::alloc(layout);

	assert!(!ptr.is_null());

	ptr
}
