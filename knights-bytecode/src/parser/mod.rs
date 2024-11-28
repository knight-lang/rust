use crate::env::Env;

mod ast;
mod parsable;
mod source;
use ast::AstNode;
use source::Source;

#[derive(Error, Debug)]
pub enum Error {
	#[error("yes")]
	Todo,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Parser<'env> {
	env: &'env Env,
	sources: Vec<Source>,
}

impl<'env> Parser<'env> {
	pub fn new(sources: impl IntoIterator<Item = Source>, env: &'env Env) -> Self {
		Self { env, sources: sources.into_iter().collect() }
	}

	pub fn parse(&self) -> Result<AstNode> {
		todo!()
	}
}
