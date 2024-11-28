use crate::{env::Env, Result};

pub mod boolean;
pub mod integer;
pub mod list;
pub mod string;
pub use boolean::{Boolean, ToBoolean};
pub use integer::{Integer, ToInteger};
pub use list::{List, ToList};
pub use string::{KString, ToString};

// Todo: more
#[derive(Debug, Clone)]
pub enum Value {
	Null,
	Boolean(bool),
	Integer(Integer),
}

impl Value {
	pub fn to_boolean(&self, env: &Env) -> Result<bool> {
		todo!()
	}

	pub fn to_string(&self, env: &Env) -> Result<KString> {
		match self {
			Self::Null => Ok(KString::new_unvalidated("")),
			Self::Boolean(true) => Ok(KString::new_unvalidated("true")),
			Self::Boolean(false) => Ok(KString::new_unvalidated("false")),
			Self::Integer(num) => Ok(KString::new(num.to_string(), env.opts())?),
		}
	}

	pub fn to_integer(&self, env: &Env) -> Result<Integer> {
		todo!()
	}

	// pub fn to_list(&self, env: &Env) -> Result<bool> {
	// 	todo!()
	// }
}
