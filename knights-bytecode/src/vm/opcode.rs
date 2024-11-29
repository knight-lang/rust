const fn opcode(id: u8, arity: u8, takes_offset: bool) -> u8 {
	(arity << 5) | if takes_offset { 1 } else { 0 } | (id << 1)
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Opcode {
	// Builtins
	PushConstant = opcode(0, 0, true),
	Jump = opcode(1, 0, true),
	JumpIfTrue = opcode(2, 1, true),
	JumpIfFalse = opcode(3, 1, true),
	GetVar = opcode(4, 0, true),
	SetVar = opcode(5, 0, true),    // no opcode cause top of stack
	SetVarPop = opcode(6, 1, true), // same as setvar but it pips

	// Arity 0
	Prompt = opcode(0, 0, false),
	Random = opcode(1, 0, false),
	Dup = opcode(2, 0, false), // doesnt have an arity cause that pops
	Return = opcode(3, 0, false),

	// Arity 1
	Call = opcode(0, 1, false),
	Quit = opcode(1, 1, false),
	Dump = opcode(2, 1, false),
	Output = opcode(3, 1, false),
	Length = opcode(4, 1, false),
	Not = opcode(5, 1, false),
	Negate = opcode(6, 1, false),
	Ascii = opcode(7, 1, false),
	Box = opcode(8, 1, false),
	Head = opcode(9, 1, false),
	Tail = opcode(10, 1, false),
	Pop = opcode(11, 1, false),

	// Arity 2
	Add = opcode(0, 2, false),
	Sub = opcode(1, 2, false),
	Mul = opcode(2, 2, false),
	Div = opcode(3, 2, false),
	Mod = opcode(4, 2, false),
	Pow = opcode(5, 2, false),
	Lth = opcode(6, 2, false),
	Gth = opcode(7, 2, false),
	Eql = opcode(8, 2, false),

	// Arity 3
	Get = opcode(0, 3, false),

	// Arity 4
	Set = opcode(0, 4, false),
}

impl Opcode {
	pub unsafe fn from_byte_unchecked(byte: u8) -> Self {
		if !cfg!(debug_assertions) {
			// Safety: u8 and opcode have same reprs, and caller ensures it's a
			// valid opcode.
			return unsafe { std::mem::transmute::<u8, Opcode>(byte) };
		}

		match byte {
			_ if byte == Self::PushConstant as u8 => Self::PushConstant,
			_ if byte == Self::Jump as u8 => Self::Jump,
			_ if byte == Self::JumpIfTrue as u8 => Self::JumpIfTrue,
			_ if byte == Self::JumpIfFalse as u8 => Self::JumpIfFalse,
			_ if byte == Self::GetVar as u8 => Self::GetVar,
			_ if byte == Self::SetVar as u8 => Self::SetVar,
			_ if byte == Self::SetVarPop as u8 => Self::SetVarPop,
			_ if byte == Self::Prompt as u8 => Self::Prompt,
			_ if byte == Self::Random as u8 => Self::Random,
			_ if byte == Self::Dup as u8 => Self::Dup,
			_ if byte == Self::Call as u8 => Self::Call,
			_ if byte == Self::Quit as u8 => Self::Quit,
			_ if byte == Self::Dump as u8 => Self::Dump,
			_ if byte == Self::Output as u8 => Self::Output,
			_ if byte == Self::Length as u8 => Self::Length,
			_ if byte == Self::Not as u8 => Self::Not,
			_ if byte == Self::Negate as u8 => Self::Negate,
			_ if byte == Self::Ascii as u8 => Self::Ascii,
			_ if byte == Self::Box as u8 => Self::Box,
			_ if byte == Self::Head as u8 => Self::Head,
			_ if byte == Self::Tail as u8 => Self::Tail,
			_ if byte == Self::Pop as u8 => Self::Pop,
			_ if byte == Self::Add as u8 => Self::Add,
			_ if byte == Self::Sub as u8 => Self::Sub,
			_ if byte == Self::Mul as u8 => Self::Mul,
			_ if byte == Self::Div as u8 => Self::Div,
			_ if byte == Self::Mod as u8 => Self::Mod,
			_ if byte == Self::Pow as u8 => Self::Pow,
			_ if byte == Self::Lth as u8 => Self::Lth,
			_ if byte == Self::Gth as u8 => Self::Gth,
			_ if byte == Self::Eql as u8 => Self::Eql,
			_ if byte == Self::Get as u8 => Self::Get,
			_ if byte == Self::Set as u8 => Self::Set,
			_ if byte == Self::Return as u8 => Self::Return,
			_ => unreachable!("invalid opcode: {byte:08b}"),
		}
	}

	pub const MAX_ARITY: usize = 4;

	pub const fn arity(self) -> usize {
		((self as u8) >> 5) as usize
	}

	pub const fn takes_offset(self) -> bool {
		(self as u8) & 1 != 0
	}
}
