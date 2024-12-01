use crate::options::Options;

#[derive(Default)]
pub struct Environment {
	opts: Options,
}

impl Environment {
	pub fn opts(&self) -> &Options {
		&self.opts
	}
}
