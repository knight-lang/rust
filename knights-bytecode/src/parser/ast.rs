use crate::env::Env;
use crate::strings::StringSlice;
use std::ops::Range;

use super::source::Source;

pub struct AstNode<'source, 'env> {
	env: &'env Env,
	source: &'source Source,
	range: Range<usize>,
	kind: AstNodeKind,
}

pub enum AstNodeKind {
	Null,
	Boolean,
	EmptyArray,
	String,
	Integer,
	Identifier,
	SymbolFn,
	WordFn,
}
