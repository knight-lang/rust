use crate::env::Env;

mod parsable;
mod source;

use source::Source;

pub struct Parser<'env> {
	env: &'env Env,
	sources: Vec<Source>,
}

impl<'env> Parser<'env> {
	pub fn new(sources: impl IntoIterator<Item = Source>, env: &'env Env) -> Self {
		Self { env, sources: sources.into_iter().collect() }
	}
}
