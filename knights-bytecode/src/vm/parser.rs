use std::path::Path;

use crate::Environment;

use super::{Builder, Program};

#[derive(Debug)]
pub struct SourceLocation<'filename> {
	filename: Option<&'filename Path>,
	line: usize,
}

pub struct Parser<'env, 'filename, 'expr> {
	environment: &'env mut Environment,
	filename: Option<&'filename Path>,
	contents: &'expr str,
	builder: Builder<'filename>,
}

impl<'env, 'filename, 'expr> Parser<'env, 'filename, 'expr> {
	pub fn with_filename(
		environment: &'env mut Environment,
		path: &'filename Path,
		contents: &'expr str,
	) -> Self {
		Self { environment, filename: Some(path), contents, builder: Builder::default() }
	}

	pub fn new(environment: &'env mut Environment, contents: &'expr str) -> Self {
		Self { environment, filename: None, contents, builder: Builder::default() }
	}

	pub fn parse(&mut self) -> ! {
		todo!()
	}
}
