#![deny(missing_docs)]
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

/// NamedType is a trait that indicates a type is
pub trait NamedType {
	/// The name of a type.
	const TYPENAME: &'static str;
}
