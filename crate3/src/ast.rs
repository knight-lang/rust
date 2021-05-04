use crate::{Result, Value, Environment};

#[derive(Debug, Clone)]
pub struct Ast {

}

impl Ast {
	pub fn into_raw(self) -> *const () {
		todo!()
	}

	pub unsafe fn from_raw(raw: *const ()) -> Self {
		todo!()
	}

	pub fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Value> {
		todo!()
	}
}