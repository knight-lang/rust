#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Opcode {
	// Builtins
	PushConstant,
	Jump,
	Pop,
	Dup,
	JumpIfTrue,
	JumpIfFalse,
	GetVar,
	SetVar,

	// Arity 0
	Prompt,
	Random,

	Call,
	Quit,
	Dump,
	Output,
	Length,
	Not,
	Negate,
	Ascii,
	Box,
	Head,
	Tail,

	// Arity 2
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Pow,
	Lth,
	Gth,
	Eql,

	// Arity 3
	Get,

	// Arity 4
	Set,
}

impl Opcode {
	pub fn arity(&self) -> usize {
		match self {
			// Builtins
			Self::PushConstant => 0,
			Self::Jump => 0,
			Self::Pop => 1,
			Self::Dup => 0,
			Self::JumpIfTrue => 1,
			Self::JumpIfFalse => 1,
			Self::GetVar => 0,
			Self::SetVar => 1,

			// Arity 0
			Self::Prompt | Self::Random => 0,

			// Arity 1
			Self::Call
			| Self::Quit
			| Self::Dump
			| Self::Output
			| Self::Length
			| Self::Not
			| Self::Negate
			| Self::Ascii
			| Self::Box
			| Self::Head
			| Self::Tail => 1,

			// Arity 2
			Self::Add
			| Self::Sub
			| Self::Mul
			| Self::Div
			| Self::Mod
			| Self::Pow
			| Self::Lth
			| Self::Gth
			| Self::Eql => 2,

			// Arity 3
			Self::Get => 3,

			// Arity 4
			Self::Set => 4,
		}
	}
}
