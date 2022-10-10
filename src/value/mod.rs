//! Types relating to [`Value`]s.

mod boolean;
#[cfg(feature = "custom-types")]
pub mod custom;
mod integer;
mod list;
mod null;
pub mod text;
mod value;

pub use boolean::{Boolean, ToBoolean};
#[cfg(feature = "custom-types")]
pub use custom::{Custom, CustomType};
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
