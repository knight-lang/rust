use crate::{Environment, Result, Number, Boolean, Text};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Value(u64);

pub trait ValueKind : Debug + Into<Value> {
	fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Value>;

	fn to_number(&self, env: &mut Environment<'_, '_, '_>) -> Result<Number>;
	fn to_text(&self, env: &mut Environment<'_, '_, '_>) -> Result<Text>;
	fn to_boolean(&self, env: &mut Environment<'_, '_, '_>) -> Result<Boolean>;
}

pub trait Idempotent : Debug + Clone + Into<Value> + Into<Number> + Into<Text> + Into<Boolean> {}

impl<T: Idempotent> ValueKind for T {
	fn run(&self, _: &mut Environment<'_, '_, '_>) -> Result<Value> {
		Ok(self.clone().into())
	}

	fn to_number(&self, _: &mut Environment<'_, '_, '_>) -> Result<Number> {
		Ok(self.clone().into())
	}

	fn to_text(&self, _: &mut Environment<'_, '_, '_>) -> Result<Text> {
		Ok(self.clone().into())
	}

	fn to_boolean(&self, _: &mut Environment<'_, '_, '_>) -> Result<Boolean> {
		Ok(self.clone().into())
	}
}

impl Value {
	pub const unsafe fn from_bytes(bytes: u64) -> Self {
		Self(bytes)
	}

	pub const fn bytes(&self) -> u64 {
		self.0
	}
}

// a value is not a value kind, so it doesnt implement it (even though it has the same functions)
impl Value {
	pub fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Self> {
		let _ = env;
		todo!()
	}

	pub fn to_number(&self, env: &mut Environment<'_, '_, '_>) -> Result<Number> {
		let _ = env;
		todo!()
	}

	pub fn to_boolean(&self, env: &mut Environment<'_, '_, '_>) -> Result<Boolean> {
		let _ = env;
		todo!()
	}

	pub fn to_text(&self, env: &mut Environment<'_, '_, '_>) -> Result<Text> {
		let _ = env;
		todo!()
	}
}
