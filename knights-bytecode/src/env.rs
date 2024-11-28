use crate::options::Options;

#[derive(Default)]
pub struct Env {
	opts: Options,
}

impl Env {
	pub fn opts(&self) -> &Options {
		&self.opts
	}
}
