//! Types relating to [`Value`]s.
// #![warn(missing_docs)]

mod boolean;
mod integer;
mod list;
mod null;
pub mod text;
mod value;

#[cfg(feature = "custom-types")]
#[cfg_attr(docsrs, doc(cfg(feature = "custom-types")))]
mod custom;

pub use boolean::{Boolean, ToBoolean};
#[cfg(feature = "custom-types")]
pub use custom::{Custom, CustomType};
#[doc(inline)]
pub use integer::{Integer, ToInteger};
pub use list::{List, ToList};
pub use null::Null;
#[doc(inline)]
pub use text::{Text, TextSlice, ToText};
pub use value::Value;

/// A trait indicating a type has a name.
pub trait NamedType {
	/// The name of a type.
	const TYPENAME: &'static str;
}

/// A trait indicating a type can be run.
pub trait Runnable {
	/// Runs `self`.
	fn run(&self, env: &mut crate::env::Environment) -> crate::Result<Value>;
}
