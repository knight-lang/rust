use crate::value::Value;
use crate::{Environment, Result};

#[derive(Clone, Default)]
pub struct List(Option<Box<[Value]>>);

pub trait ToList {
	fn to_list(&self, env: &mut Environment) -> Result<List>;
}
