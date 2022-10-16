//! Types relating to [`Value`]s.
// #![warn(missing_docs)]

mod boolean;
pub mod integer;
mod list;
mod null;
pub mod text;
mod value;

#[cfg(feature = "custom-types")]
#[cfg_attr(docsrs, doc(cfg(feature = "custom-types")))]
pub mod custom;

pub use boolean::{Boolean, ToBoolean};
pub use integer::{IntType, Integer, ToInteger};
pub use list::{List, ToList};
pub use null::Null;
pub use text::*;
pub use value::Value;

#[cfg(feature = "custom-types")]
pub use custom::{Custom, CustomType};

/// A trait indicating a type has a name.
pub trait NamedType {
	/// The name of a type.
	const TYPENAME: &'static str;
}

/// A trait indicating a type can be run.
pub trait Runnable<I, E> {
	/// Runs `self`.
	fn run(&self, env: &mut crate::Environment<I, E>) -> crate::Result<Value<I, E>>;
}
