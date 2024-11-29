use std::path::Path;

#[derive(Debug)]
pub struct SourceLocation<'file> {
	filename: Option<&'file Path>,
	line: usize,
}

pub struct Parser {}
