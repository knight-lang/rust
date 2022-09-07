//! Types relating to [`Value`]s.

mod boolean;
mod integer;
mod list;
mod null;
pub mod text;
mod value;

pub use boolean::{Boolean, ToBoolean};
pub use integer::{Integer, ToInteger};
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
pub trait Runnable<'e> {
	/// Runs `self`.
	fn run(&self, env: &mut crate::Environment<'e>) -> crate::Result<Value<'e>>;
}
