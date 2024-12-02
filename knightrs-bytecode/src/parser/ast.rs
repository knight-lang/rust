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
	// Simple
	Literal(Value),
	Variable(VariableName),

	// Compound stuff
	While(Box<Ast>, Box<Ast>),
	If(Box<Ast>, Box<Ast>, Box<Ast>),
	While(Box<Ast>, Box<Ast>, Box<Ast>),

	// Functions
	Nullary(NullaryFn),
	Unary(UnaryFn, Box<Ast>),
	Binary(BinaryFn, Box<Ast>, Box<Ast>),
	Ternary(TernaryFn, Box<Ast>, Box<Ast>, Box<Ast>),
	Quaternary(QuaternaryFn, Box<Ast>, Box<Ast>, Box<Ast>, Box<Ast>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NullaryFn {
	Prompt = Opcode::Prompt as u8,
	Random = Opcode::Random as u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UnaryFn {
	Call = Opcode::Call as u8,
	Quit = Opcode::Quit as u8,
	Dump = Opcode::Dump as u8,
	Output = Opcode::Output as u8,
	Length = Opcode::Length as u8,
	Not = Opcode::Not as u8,
	Negate = Opcode::Negate as u8,
	Ascii = Opcode::Ascii as u8,
	Box = Opcode::Box as u8,
	Head = Opcode::Head as u8,
	Tail = Opcode::Tail as u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BinaryFn {
	Add = Opcode::Add as u8,
	Sub = Opcode::Sub as u8,
	Mul = Opcode::Mul as u8,
	Div = Opcode::Div as u8,
	Mod = Opcode::Mod as u8,
	Pow = Opcode::Pow as u8,
	Lth = Opcode::Lth as u8,
	Gth = Opcode::Gth as u8,
	Eql = Opcode::Eql as u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TernaryFn {
	Get = Opcode::Get as u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QuaternaryFn {
	Set = Opcode::Set as u8,
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
		match self.kind {
			AstKind::Literal(value) => {
				compiler.push_constant(value);
				Ok(())
			}
			AstKind::Variable(name) => {
				compiler.get_variable(name, opts).map_err(|err| self.loc.error(err))
			}

			AstKind::Nullary(nullary) => unsafe {
				compiler.opcode_without_offset(Opcode::from_byte_unchecked(nullary as u8));
				Ok(())
			},
			AstKind::Unary(unary, arg1) => unsafe {
				arg1.compile(compiler, opts)?;
				compiler.opcode_without_offset(Opcode::from_byte_unchecked(unary as u8));
				Ok(())
			},
			AstKind::Binary(binary, arg1, arg2) => unsafe {
				arg1.compile(compiler, opts)?;
				arg2.compile(compiler, opts)?;
				compiler.opcode_without_offset(Opcode::from_byte_unchecked(binary as u8));
				Ok(())
			},
			AstKind::Ternary(ternary, arg1, arg2, arg3) => unsafe {
				arg1.compile(compiler, opts)?;
				arg2.compile(compiler, opts)?;
				arg3.compile(compiler, opts)?;
				compiler.opcode_without_offset(Opcode::from_byte_unchecked(ternary as u8));
				Ok(())
			},
			AstKind::Quaternary(quaternary, arg1, arg2, arg3, arg4) => unsafe {
				arg1.compile(compiler, opts)?;
				arg2.compile(compiler, opts)?;
				arg3.compile(compiler, opts)?;
				arg4.compile(compiler, opts)?;
				compiler.opcode_without_offset(Opcode::from_byte_unchecked(quaternary as u8));
				Ok(())
			},
		}
	}
}
