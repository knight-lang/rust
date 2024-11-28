use crate::{env::Env, KString, Result};

pub type Integer = i64;

// Todo: more
#[derive(Debug, Clone)]
pub enum Value {
	Null,
	Boolean(bool),
	Integer(i64),
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
