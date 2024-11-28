use crate::{Environment, Result};

pub mod boolean;
pub mod integer;
pub mod list;
pub mod null;
pub mod string;
pub use boolean::{Boolean, ToBoolean};
pub use integer::{Integer, ToInteger};
pub use list::{List, ToList};
pub use null::Null;
pub use string::{KString, ToKString};

// Todo: more
#[derive(Debug, Clone)]
pub enum Value {
	Null,
	Boolean(Boolean),
	Integer(Integer),
	String(KString),
	List(List),
}

impl From<Boolean> for Value {
	fn from(b: Boolean) -> Self {
		Self::Boolean(b)
	}
}

impl From<Null> for Value {
	fn from(_: Null) -> Self {
		Self::Null
	}
}

impl From<KString> for Value {
	fn from(string: KString) -> Self {
		Self::String(string)
	}
}

impl From<List> for Value {
	fn from(list: List) -> Self {
		Self::List(list)
	}
}

impl ToBoolean for Value {
	fn to_boolean(&self, env: &mut Environment) -> Result<Boolean> {
		match self {
			Self::Null => Null.to_boolean(env),
			Self::Boolean(boolean) => boolean.to_boolean(env),
			Self::Integer(integer) => integer.to_boolean(env),
			Self::String(string) => string.to_boolean(env),
			Self::List(list) => list.to_boolean(env),
		}
	}
}

impl ToInteger for Value {
	fn to_integer(&self, env: &mut Environment) -> Result<Integer> {
		match self {
			Self::Null => Null.to_integer(env),
			Self::Boolean(boolean) => boolean.to_integer(env),
			Self::Integer(integer) => integer.to_integer(env),
			Self::String(string) => string.to_integer(env),
			Self::List(list) => list.to_integer(env),
		}
	}
}

impl ToKString for Value {
	fn to_kstring(&self, env: &mut Environment) -> Result<KString> {
		match self {
			Self::Null => Null.to_kstring(env),
			Self::Boolean(boolean) => boolean.to_kstring(env),
			Self::Integer(integer) => integer.to_kstring(env),
			Self::String(string) => string.to_kstring(env),
			Self::List(list) => list.to_kstring(env),
		}
	}
}

impl ToList for Value {
	fn to_list(&self, env: &mut Environment) -> Result<List> {
		match self {
			Self::Null => Null.to_list(env),
			Self::Boolean(boolean) => boolean.to_list(env),
			Self::Integer(integer) => integer.to_list(env),
			Self::String(string) => string.to_list(env),
			Self::List(list) => list.to_list(env),
		}
	}
}
