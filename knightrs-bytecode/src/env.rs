use crate::options::Options;
use crate::value::{Integer, KString};

#[derive(Default)]
pub struct Environment {
	opts: Options,
}

impl Environment {
	pub fn opts(&self) -> &Options {
		&self.opts
	}

	pub fn prompt(&mut self) -> crate::Result<Option<KString>> {
		todo!()
	}

	pub fn random(&mut self) -> crate::Result<Integer> {
		todo!()
	}
}
