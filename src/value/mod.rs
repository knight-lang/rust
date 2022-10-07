//! Types relating to [`Value`]s.

mod boolean;
pub mod integer;
mod list;
mod null;
pub mod text;
// pub mod text2;
mod value;

pub use boolean::{Boolean, ToBoolean};
pub use integer::{IntType, Integer, ToInteger};
pub use list::{List, ToList};
pub use null::Null;
pub use text::*;
pub use value::Value;

/// A trait indicating a type has a name.
pub trait NamedType {
	/// The name of a type.
	const TYPENAME: &'static str;
}

/// A trait indicating a type can be run.
pub trait Runnable<'e, E: Encoding, I: IntType> {
	/// Runs `self`.
	fn run(&self, env: &mut crate::Environment<'e, E, I>) -> crate::Result<Value<'e, E, I>>;
}
