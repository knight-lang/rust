use crate::value::Value;
use crate::vm::Opcode;
use crate::Options;
use crate::{program::Compiler, value::KString};

use super::{ParseError, SourceLocation, VariableName};

pub struct Ast {
	pub loc: SourceLocation,
	pub kind: AstKind,
}

pub enum AstKind {
	Literal(Value),
	Variable(VariableName),
	Function(Opcode, Vec<Value>),
}

impl From<Value> for AstKind {
	fn from(value: Value) -> Self {
		Self::Literal(value)
	}
}

impl Ast {
	pub fn new(kind: impl Into<AstKind>, loc: SourceLocation) -> Self {
		Self { kind: kind.into(), loc }
	}

	pub fn compile(self, compiler: &mut Compiler, opts: &Options) -> Result<(), ParseError> {
		todo!()
	}
}
