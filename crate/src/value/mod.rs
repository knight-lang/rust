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

pub trait KnightType: Into<Value> {
	const TYPENAME: &'static str;
}
